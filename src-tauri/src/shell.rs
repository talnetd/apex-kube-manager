//! Multi-window shell (spec 13.3, 13.4): one shared context for every
//! window, broadcast from Rust, plus opening and counting main windows.
//!
//! The kubeconfig's `current-context` stays the single source of truth (the
//! existing switch writes it, and kubectx users change it from outside).
//! `ContextSync` remembers the last value every window was told about; a
//! switch from any window, the tray, or an outside change emits one
//! `context-changed { context, seq }` event to all windows. `seq` only ever
//! grows, so a window can drop an out-of-order event.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::error::{AppError, Result};
use crate::kubernetes;

pub const CONTEXT_CHANGED: &str = "context-changed";
pub const WINDOWS_CHANGED: &str = "windows-changed";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ContextChanged {
    pub context: String,
    pub seq: u64,
}

#[derive(Default)]
pub struct ContextSync {
    last: Mutex<Option<String>>,
    seq: AtomicU64,
    /// Serialises kubeconfig writes (switch) with the outside-change watcher,
    /// so a watcher read can't race a switch and broadcast a stale context.
    pub lock: tokio::sync::Mutex<()>,
}

impl ContextSync {
    /// Record `context` as current. Returns the event to broadcast, or None
    /// when every window already has it.
    pub fn observe(&self, context: &str) -> Option<ContextChanged> {
        let mut last = self.last.lock().unwrap_or_else(|e| e.into_inner());
        if last.as_deref() == Some(context) {
            return None;
        }
        *last = Some(context.to_string());
        let seq = self.seq.fetch_add(1, Ordering::SeqCst) + 1;
        Some(ContextChanged {
            context: context.to_string(),
            seq,
        })
    }

    /// Set without broadcasting (startup: windows read it themselves).
    pub fn prime(&self, context: &str) {
        *self.last.lock().unwrap_or_else(|e| e.into_inner()) = Some(context.to_string());
    }

    /// What every window should be on right now, with its seq. Windows call
    /// this once after they start listening, to catch a switch they missed.
    pub fn current(&self) -> Option<ContextChanged> {
        let last = self
            .last
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()?;
        Some(ContextChanged {
            context: last,
            seq: self.seq.load(Ordering::SeqCst),
        })
    }
}

/// Tell every window (main, detail, terminal, tray) about the context.
/// Returns the state windows should end up on.
pub fn broadcast_context(app: &AppHandle, context: &str) -> ContextChanged {
    let sync = app.state::<ContextSync>();
    match sync.observe(context) {
        Some(ev) => {
            let _ = app.emit(CONTEXT_CHANGED, ev.clone());
            crate::tray::on_context_changed(app);
            ev
        }
        None => sync.current().unwrap_or(ContextChanged {
            context: context.to_string(),
            seq: 0,
        }),
    }
}

/// Switch the shared context (kubeconfig write once, here) and broadcast it.
/// The returned seq lets the caller ignore anything older.
pub async fn switch_shared_context(app: &AppHandle, context: &str) -> Result<ContextChanged> {
    let sync = app.state::<ContextSync>();
    let _guard = sync.lock.lock().await;
    kubernetes::switch_to_context(context).await?;
    Ok(broadcast_context(app, context))
}

/// Outside changes (kubectx, another tool) poll the kubeconfig every 3s,
/// once, in Rust, instead of in every window.
pub fn start_context_watcher(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Ok(ctx) = kubernetes::get_current_context_name().await {
            app.state::<ContextSync>().prime(&ctx);
        }
        let mut tick = tokio::time::interval(Duration::from_secs(3));
        loop {
            tick.tick().await;
            let sync = app.state::<ContextSync>();
            let _guard = sync.lock.lock().await;
            if let Ok(ctx) = kubernetes::get_current_context_name().await {
                let changed = {
                    let last = sync.last.lock().unwrap_or_else(|e| e.into_inner());
                    last.as_deref() != Some(ctx.as_str())
                };
                if changed {
                    // Unsettled clocks belong to the old cluster's objects.
                    crate::triage::reset_unsettled();
                    broadcast_context(&app, &ctx);
                }
            }
        }
    });
}

// ============ Main windows ============

/// `main` (from tauri.conf.json) plus `main-<n>` opened with ⌘N.
pub fn is_main_label(label: &str) -> bool {
    label == "main" || label.starts_with("main-")
}

pub fn main_window_count(app: &AppHandle) -> usize {
    app.webview_windows()
        .keys()
        .filter(|l| is_main_label(l))
        .count()
}

pub fn emit_window_count(app: &AppHandle) {
    let _ = app.emit(WINDOWS_CHANGED, main_window_count(app));
}

static NEXT_WINDOW: AtomicU64 = AtomicU64::new(2);

/// Open another main window on the shared context, starting on Pulse.
pub fn open_main_window(app: &AppHandle) -> Result<()> {
    let label = loop {
        let n = NEXT_WINDOW.fetch_add(1, Ordering::SeqCst);
        let label = format!("main-{n}");
        if app.get_webview_window(&label).is_none() {
            break label;
        }
    };
    WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
        .title("Apex")
        .inner_size(1200.0, 820.0)
        .min_inner_size(400.0, 480.0)
        .resizable(true)
        .decorations(false)
        .build()
        .map_err(|e| AppError::Custom(format!("Failed to open window: {e}")))?;
    emit_window_count(app);
    Ok(())
}

/// A window closed: stop the watches it started, and when the last main
/// window goes with the menu bar item off, quit (a hidden popover must not
/// keep a windowless app alive).
pub fn on_window_destroyed(app: &AppHandle, label: &str) {
    let owned = label.to_string();
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let n = handle
            .state::<crate::watch::WatchManager>()
            .stop_owned_by(&owned)
            .await;
        if n > 0 {
            eprintln!("[apex-kube-manager] stopped {n} watches of closed window {owned}");
        }
    });
    let label = label.to_string();
    if is_main_label(&label) {
        // The closing window can still be listed during Destroyed.
        let left = app
            .webview_windows()
            .keys()
            .filter(|l| is_main_label(l) && l.as_str() != label)
            .count();
        let _ = app.emit(WINDOWS_CHANGED, left);
        let tray_on = app.state::<crate::tray::TrayState>().is_enabled();
        if left == 0 && !tray_on {
            eprintln!("[apex-kube-manager] last window closed, menu bar item off: quitting");
            app.exit(0);
        }
    }
}

/// Focus the most recent main window, or open one if none is left (the
/// app keeps running in the menu bar after the last window closes).
pub fn focus_or_open_main(app: &AppHandle) -> Result<()> {
    let mut mains: Vec<_> = app
        .webview_windows()
        .into_iter()
        .filter(|(l, _)| is_main_label(l))
        .collect();
    mains.sort_by(|a, b| a.0.cmp(&b.0));
    match mains.pop() {
        Some((_, w)) => {
            let _ = w.unminimize();
            let _ = w.show();
            w.set_focus().map_err(|e| AppError::Custom(e.to_string()))
        }
        None => open_main_window(app),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_broadcasts_only_changes_with_growing_seq() {
        let s = ContextSync::default();
        let a = s.observe("prod").unwrap();
        assert_eq!(
            a,
            ContextChanged {
                context: "prod".into(),
                seq: 1
            }
        );
        assert_eq!(s.observe("prod"), None);
        let b = s.observe("staging").unwrap();
        assert_eq!(b.seq, 2);
        assert_eq!(s.observe("prod").unwrap().seq, 3);
    }

    #[test]
    fn prime_sets_without_an_event() {
        let s = ContextSync::default();
        s.prime("prod");
        assert_eq!(s.observe("prod"), None);
    }

    #[test]
    fn main_labels() {
        assert!(is_main_label("main"));
        assert!(is_main_label("main-3"));
        assert!(!is_main_label("detail-pod-x"));
        assert!(!is_main_label("tray"));
        assert!(!is_main_label("mainframe"));
    }
}

// ============ App menu: File > New Window, Show in Menu Bar ============

pub const MENU_NEW_WINDOW: &str = "new-window";
pub const MENU_TRAY: &str = "toggle-tray";

/// Default menus (Edit keeps copy/paste working) plus our two items at the
/// top of File, so ⌘N works from every window.
pub fn build_menu(app: &AppHandle, tray_on: bool) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
    let menu = Menu::default(app)?;
    let new_window = MenuItem::with_id(
        app,
        MENU_NEW_WINDOW,
        "New Window",
        true,
        Some("CmdOrCtrl+N"),
    )?;
    let tray = CheckMenuItem::with_id(
        app,
        MENU_TRAY,
        "Show in Menu Bar",
        true,
        tray_on,
        None::<&str>,
    )?;
    let file = menu
        .items()?
        .into_iter()
        .filter_map(|i| i.as_submenu().cloned())
        .find(|s| s.text().map(|t| t == "File").unwrap_or(false));
    match file {
        Some(file) => {
            file.insert(&new_window, 0)?;
            file.insert(&tray, 1)?;
            file.insert(&PredefinedMenuItem::separator(app)?, 2)?;
        }
        None => {
            let sub = tauri::menu::Submenu::with_items(app, "File", true, &[&new_window, &tray])?;
            menu.insert(&sub, 1)?;
        }
    }
    Ok(menu)
}

/// Keep the menu check mark in step when the popover turns the item off.
pub fn sync_tray_menu(app: &AppHandle, on: bool) {
    if let Some(menu) = app.menu() {
        if let Some(item) = find_item(&menu.items().unwrap_or_default(), MENU_TRAY) {
            if let Some(check) = item.as_check_menuitem() {
                let _ = check.set_checked(on);
            }
        }
    }
}

fn find_item(
    items: &[tauri::menu::MenuItemKind<tauri::Wry>],
    id: &str,
) -> Option<tauri::menu::MenuItemKind<tauri::Wry>> {
    for i in items {
        if i.id() == id {
            return Some(i.clone());
        }
        if let Some(sub) = i.as_submenu() {
            if let Some(found) = find_item(&sub.items().unwrap_or_default(), id) {
                return Some(found);
            }
        }
    }
    None
}
