use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

/// Manages PTY sessions for terminal connections
pub struct PtyManager {
    sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

struct PtySession {
    pair: PtyPair,
    writer: Box<dyn Write + Send>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawn a new PTY session for kubectl exec
    pub fn spawn_session(
        &self,
        app: AppHandle,
        namespace: &str,
        pod_name: &str,
        container: Option<&str>,
        shell: Option<&str>,
    ) -> Result<String, String> {
        let pty_system = native_pty_system();

        // Create PTY with initial size
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        // Build kubectl exec command
        let mut cmd = CommandBuilder::new("kubectl");
        cmd.args(["exec", "-it", pod_name, "-n", namespace]);

        if let Some(c) = container {
            cmd.args(["-c", c]);
        }

        // Use specified shell or default to /bin/sh
        // Common shells: /bin/sh, /bin/bash, /bin/ash, /bin/zsh
        let shell_path = shell.unwrap_or("/bin/sh");
        cmd.args(["--", shell_path]);

        // Spawn the command in the PTY
        let mut child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn kubectl: {}", e))?;

        // Get writer for sending input
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to get PTY writer: {}", e))?;

        // Get reader for receiving output
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to get PTY reader: {}", e))?;

        // Generate session ID
        let session_id = Uuid::new_v4().to_string();
        let session_id_for_thread = session_id.clone();

        // Clone app handle for reader thread
        let app_for_reader = app;

        // Store session
        {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(
                session_id.clone(),
                PtySession {
                    pair,
                    writer,
                },
            );
        }

        // Spawn thread to read PTY output and emit events
        let sessions_clone = Arc::clone(&self.sessions);
        thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        // EOF - process exited
                        let _ = app_for_reader.emit(&format!("pty-exit-{}", session_id_for_thread), ());
                        break;
                    }
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                        let _ = app_for_reader.emit(&format!("pty-data-{}", session_id_for_thread), data);
                    }
                    Err(e) => {
                        tracing::error!("PTY read error: {}", e);
                        let _ = app_for_reader.emit(&format!("pty-error-{}", session_id_for_thread), e.to_string());
                        break;
                    }
                }
            }

            // Clean up session
            let mut sessions = sessions_clone.lock().unwrap();
            sessions.remove(&session_id_for_thread);
        });

        // Spawn thread to wait for process exit (reader thread handles exit event on EOF)
        thread::spawn(move || {
            let _ = child.wait();
        });

        Ok(session_id)
    }

    /// Write data to a PTY session
    pub fn write_to_session(&self, session_id: &str, data: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session
                .writer
                .write_all(data.as_bytes())
                .map_err(|e| format!("Failed to write to PTY: {}", e))?;
            session
                .writer
                .flush()
                .map_err(|e| format!("Failed to flush PTY: {}", e))?;
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Resize a PTY session
    pub fn resize_session(&self, session_id: &str, rows: u16, cols: u16) -> Result<(), String> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            session
                .pair
                .master
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|e| format!("Failed to resize PTY: {}", e))?;
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Close a PTY session
    pub fn close_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap();
        if sessions.remove(session_id).is_some() {
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}
