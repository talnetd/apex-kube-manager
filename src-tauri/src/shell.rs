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
        let mut last_mtime = kubeconfig_mtime();
        loop {
            tick.tick().await;
            // Any edit (new token, renamed cluster) invalidates cached clients.
            let mtime = kubeconfig_mtime();
            if mtime != last_mtime {
                last_mtime = mtime;
                crate::tray::invalidate_clients(&app);
            }
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

fn kubeconfig_mtime() -> Option<std::time::SystemTime> {
    std::fs::metadata(kubernetes::get_kubeconfig_path_for_portforward())
        .and_then(|m| m.modified())
        .ok()
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

/// Closing this main window quits the app (it's the last one, the menu bar
/// item is off). Quitting ends running terminal sessions, so ask first when
/// any are open; detail windows just close with it (pure, unit-tested).
pub fn quit_needs_confirm(other_main_windows: usize, tray_on: bool, terminals: usize) -> bool {
    other_main_windows == 0 && !tray_on && terminals > 0
}

pub const CONFIRM_QUIT: &str = "confirm-quit";

/// A close held for the terminal confirm. It can't trap the user: a second
/// close within `CLOSE_AGAIN` goes through, and if the window doesn't
/// acknowledge the prompt within `ACK_WITHIN` (still starting, hung) the
/// close goes through on its own.
struct QuitHold {
    label: String,
    at: std::time::Instant,
    acked: bool,
}

static QUIT_HOLD: Mutex<Option<QuitHold>> = Mutex::new(None);
const CLOSE_AGAIN: Duration = Duration::from_secs(3);
const ACK_WITHIN: Duration = Duration::from_millis(1500);

/// What to do with a close request, given a hold already pending (pure,
/// unit-tested): `true` = hold it and ask.
pub fn should_hold(needs_confirm: bool, pending: Option<(&str, Duration)>, label: &str) -> bool {
    if !needs_confirm {
        return false;
    }
    !matches!(pending, Some((l, since)) if l == label && since < CLOSE_AGAIN)
}

/// CloseRequested on a main window: hold it and ask that window to confirm
/// when quitting would end terminal sessions. Returns true when held.
pub fn hold_close_for_terminals(app: &AppHandle, label: &str) -> bool {
    let others = app
        .webview_windows()
        .keys()
        .filter(|l| is_main_label(l) && l.as_str() != label)
        .count();
    let terminals = app.state::<crate::pty::PtyManager>().live_count();
    let tray_on = app.state::<crate::tray::TrayState>().is_enabled();
    let needs = quit_needs_confirm(others, tray_on, terminals);

    let mut hold = QUIT_HOLD.lock().unwrap_or_else(|e| e.into_inner());
    let pending = hold.as_ref().map(|h| (h.label.as_str(), h.at.elapsed()));
    if !should_hold(needs, pending, label) {
        *hold = None;
        return false;
    }
    *hold = Some(QuitHold {
        label: label.to_string(),
        at: std::time::Instant::now(),
        acked: false,
    });
    drop(hold);
    let _ = app.emit_to(label, CONFIRM_QUIT, terminals);

    // No ack in time: the prompt can't show, so don't swallow the close.
    let (app, label) = (app.clone(), label.to_string());
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(ACK_WITHIN).await;
        let expired = {
            let mut hold = QUIT_HOLD.lock().unwrap_or_else(|e| e.into_inner());
            let expired = matches!(&*hold, Some(h) if h.label == label && !h.acked);
            if expired {
                *hold = None;
            }
            expired
        };
        if expired {
            eprintln!("[apex-kube-manager] quit prompt not shown in {label}: closing anyway");
            if let Some(w) = app.get_webview_window(&label) {
                let _ = w.destroy();
            }
        }
    });
    true
}

/// The window showed the quit prompt (keeps the hold until the user picks).
pub fn ack_quit_prompt(label: &str) {
    if let Some(h) = QUIT_HOLD.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        if h.label == label {
            h.acked = true;
        }
    }
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
    fn quit_confirm_only_when_terminals_would_die() {
        assert!(quit_needs_confirm(0, false, 2));
        assert!(
            !quit_needs_confirm(0, false, 0),
            "no terminals: quit as before"
        );
        assert!(
            !quit_needs_confirm(1, false, 2),
            "another main window stays"
        );
        assert!(
            !quit_needs_confirm(0, true, 2),
            "menu bar item keeps the app running"
        );
    }

    #[test]
    fn a_second_close_or_another_window_is_never_held() {
        assert!(should_hold(true, None, "main"), "first close asks");
        assert!(
            !should_hold(true, Some(("main", Duration::from_millis(800))), "main"),
            "closing again goes through"
        );
        assert!(
            should_hold(true, Some(("main", Duration::from_secs(10))), "main"),
            "a much later close asks again"
        );
        assert!(
            should_hold(true, Some(("main-2", Duration::from_millis(100))), "main"),
            "a hold on another window doesn't count"
        );
        assert!(!should_hold(false, None, "main"), "nothing to confirm");
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
