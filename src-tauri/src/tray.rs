//! macOS menu bar item (spec 13.5): an icon with the worst cluster's status
//! dot, and a small popover window listing contexts.
//!
//! Polling stays light: the current context gets a health summary from the
//! triage verdicts (workloads + nodes), every other context only a cheap
//! reachability probe (`/version`), every 60s, and only while the menu bar
//! icon is enabled. Results are cached here and pushed to the popover with
//! a `tray-status` event.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::image::Image;
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

use crate::error::{AppError, Result};
use crate::kubernetes;
use crate::triage::{Severity, Status, Verdict};

pub const TRAY_STATUS: &str = "tray-status";
const TRAY_ID: &str = "apex-tray";
const POPOVER: &str = "tray";
const POLL_EVERY: Duration = Duration::from_secs(60);
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

// ============ Status model (pure, unit-tested) ============

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Dot {
    /// Not checked yet.
    Grey,
    Green,
    Amber,
    Red,
}

/// Critical verdicts make a cluster red; any other problem (warn, or fail
/// with error/warning severity) makes it amber.
pub fn count_issues<'a>(verdicts: impl IntoIterator<Item = &'a Verdict>) -> (u32, u32) {
    let (mut critical, mut warning) = (0, 0);
    for v in verdicts {
        match (&v.status, &v.severity) {
            (Status::Fail, Some(Severity::Critical)) => critical += 1,
            (Status::Warn | Status::Fail, _) => warning += 1,
            _ => {}
        }
    }
    (critical, warning)
}

/// Red when unreachable or critical, amber with issues, green when connected,
/// grey before the first check. A non-current context only has reachability.
pub fn dot_for(reachable: Option<bool>, critical: u32, warning: u32) -> Dot {
    match reachable {
        None => Dot::Grey,
        Some(false) => Dot::Red,
        Some(true) if critical > 0 => Dot::Red,
        Some(true) if warning > 0 => Dot::Amber,
        Some(true) => Dot::Green,
    }
}

/// The menu bar icon shows the worst cluster.
pub fn worst(statuses: &[ClusterStatus]) -> Dot {
    statuses.iter().map(|s| s.dot).max().unwrap_or(Dot::Grey)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub context: String,
    pub cluster: String,
    pub current: bool,
    pub dot: Dot,
    pub reachable: Option<bool>,
    /// Only the current context gets counts; others are reachability only.
    pub critical: u32,
    pub warning: u32,
    pub detail: bool,
    pub error: Option<String>,
    /// Not checked in the background on purpose (signs in with a plugin).
    #[serde(default)]
    pub skipped: bool,
}

// ============ Which contexts may be probed in the background ============

/// True when a context's user signs in through an exec plugin (aws, gcloud,
/// kubelogin, ...) or a legacy auth provider (OIDC). Those plugins can open
/// a browser, prompt on the terminal, or take seconds; Apex never runs them
/// in the background, only for the context the user is working in.
pub fn uses_plugin_auth(cfg: &kube::config::Kubeconfig, context: &str) -> bool {
    let user = cfg
        .contexts
        .iter()
        .find(|c| c.name == context)
        .and_then(|c| c.context.as_ref())
        .and_then(|c| c.user.clone());
    let Some(user) = user else { return false };
    cfg.auth_infos
        .iter()
        .find(|a| a.name == user)
        .and_then(|a| a.auth_info.as_ref())
        .is_some_and(|a| a.exec.is_some() || a.auth_provider.is_some())
}

fn read_kubeconfig() -> Option<kube::config::Kubeconfig> {
    kube::config::Kubeconfig::read_from(kubernetes::get_kubeconfig_path_for_portforward()).ok()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TraySnapshot {
    pub enabled: bool,
    pub clusters: Vec<ClusterStatus>,
    /// RFC 3339 time of the last finished poll.
    pub checked_at: Option<String>,
}

// ============ Icon (drawn, no asset files) ============

const ICON: u32 = 44; // 22pt @2x

/// RGBA icon: the Apex triangle, plus a status dot bottom-right for amber or
/// red. Healthy/unchecked is a template image (black, macOS tints it for a
/// light or dark menu bar); with a dot the icon can't be a template (it
/// would lose the colour), so the triangle is a mid grey that reads on both.
pub fn draw_icon(dot: Dot) -> (Vec<u8>, bool) {
    let template = matches!(dot, Dot::Green | Dot::Grey);
    let n = ICON as usize;
    let mut px = vec![0u8; n * n * 4];
    let tri = if template {
        [0, 0, 0, 255]
    } else {
        [142, 142, 147, 255]
    };
    let dot_rgb = match dot {
        Dot::Red => [229, 72, 77, 255],
        _ => [255, 178, 36, 255],
    };
    let (cx, cy, r) = (34.0f32, 34.0f32, 8.0f32);
    for y in 0..n {
        for x in 0..n {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let i = (y * n + x) * 4;
            let d = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt();
            if !template && d <= r {
                px[i..i + 4].copy_from_slice(&dot_rgb);
                continue;
            }
            // Keep a transparent ring between the dot and the triangle.
            if !template && d <= r + 2.5 {
                continue;
            }
            // Triangle: apex (22, 6), base y = 38 from x = 6 to 38.
            let (top, bottom) = (6.0f32, 38.0f32);
            if fy >= top && fy <= bottom {
                let half = (fy - top) / (bottom - top) * 16.0;
                if (fx - 22.0).abs() <= half {
                    px[i..i + 4].copy_from_slice(&tri);
                }
            }
        }
    }
    (px, template)
}

// ============ State ============

pub struct TrayState {
    enabled: AtomicBool,
    snapshot: Mutex<TraySnapshot>,
    icon: Mutex<Option<TrayIcon>>,
    /// Bumped to stop an old poll loop when the tray is turned off.
    epoch: std::sync::atomic::AtomicU64,
    /// One sweep at a time; a request during a sweep re-runs it once after.
    sweeping: AtomicBool,
    rerun: AtomicBool,
    /// One client per static-auth context (no plugin, so safe to reuse).
    clients: Mutex<std::collections::HashMap<String, kube::Client>>,
}

impl TrayState {
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            snapshot: Mutex::new(TraySnapshot::default()),
            icon: Mutex::new(None),
            epoch: std::sync::atomic::AtomicU64::new(0),
            sweeping: AtomicBool::new(false),
            rerun: AtomicBool::new(false),
            clients: Mutex::new(std::collections::HashMap::new()),
        }
    }
}

/// Start a sweep unless one is running (then ask it to run once more).
/// Returns true when the caller should sweep (pure, unit-tested).
pub fn begin_sweep(sweeping: &AtomicBool, rerun: &AtomicBool) -> bool {
    if sweeping.swap(true, Ordering::SeqCst) {
        rerun.store(true, Ordering::SeqCst);
        return false;
    }
    true
}

impl TrayState {
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }
}

fn state(app: &AppHandle) -> tauri::State<'_, TrayState> {
    app.state::<TrayState>()
}

pub fn snapshot(app: &AppHandle) -> TraySnapshot {
    let s = state(app);
    let mut snap = s.snapshot.lock().unwrap_or_else(|e| e.into_inner()).clone();
    snap.enabled = s.enabled.load(Ordering::SeqCst);
    snap
}

// ============ Persistence (one small file in the app config dir) ============

#[derive(Serialize, Deserialize)]
struct TrayPrefs {
    enabled: bool,
}

fn prefs_path(app: &AppHandle) -> Option<std::path::PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| d.join("tray.json"))
}

/// Default on (macOS only builds the item at all).
pub fn load_enabled(app: &AppHandle) -> bool {
    prefs_path(app)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str::<TrayPrefs>(&s).ok())
        .map(|p| p.enabled)
        .unwrap_or(true)
}

fn save_enabled(app: &AppHandle, enabled: bool) {
    if let Some(p) = prefs_path(app) {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(
            p,
            serde_json::to_string(&TrayPrefs { enabled }).unwrap_or_default(),
        );
    }
}

// ============ Polling ============

async fn current_summary(context: &str) -> std::result::Result<(u32, u32), String> {
    let work = async {
        let client = kubernetes::create_client_for_context(context).await?;
        let (d, s, ds, j, n) = tokio::try_join!(
            kubernetes::list_deployments(&client, None),
            kubernetes::list_statefulsets(&client, None),
            kubernetes::list_daemonsets(&client, None),
            kubernetes::list_jobs(&client, None),
            kubernetes::list_nodes(&client),
        )?;
        let verdicts = d
            .iter()
            .map(|x| &x.verdict)
            .chain(s.iter().map(|x| &x.verdict))
            .chain(ds.iter().map(|x| &x.verdict))
            .chain(j.iter().map(|x| &x.verdict))
            .chain(n.iter().map(|x| &x.verdict));
        Ok::<_, AppError>(count_issues(verdicts))
    };
    match tokio::time::timeout(Duration::from_secs(15), work).await {
        Ok(Ok(c)) => Ok(c),
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("Timed out".into()),
    }
}

async fn reachable(app: &AppHandle, context: &str) -> std::result::Result<(), String> {
    let cached = state(app)
        .clients
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(context)
        .cloned();
    let probe = async {
        let client = match cached {
            Some(c) => c,
            None => {
                let c = kubernetes::create_client_for_context(context).await?;
                state(app)
                    .clients
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(context.to_string(), c.clone());
                c
            }
        };
        client.apiserver_version().await?;
        Ok::<_, AppError>(())
    };
    match tokio::time::timeout(PROBE_TIMEOUT, probe).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("Timed out".into()),
    }
}

async fn check_one(
    app: AppHandle,
    context: String,
    cluster: String,
    current: bool,
    plugin: bool,
) -> ClusterStatus {
    if !current && plugin {
        return ClusterStatus {
            dot: Dot::Grey,
            context,
            cluster,
            current,
            reachable: None,
            critical: 0,
            warning: 0,
            detail: false,
            error: None,
            skipped: true,
        };
    }
    let (reachable, critical, warning, error) = if current {
        match current_summary(&context).await {
            Ok((c, w)) => (Some(true), c, w, None),
            Err(e) => (Some(false), 0, 0, Some(e)),
        }
    } else {
        match reachable(&app, &context).await {
            Ok(()) => (Some(true), 0, 0, None),
            Err(e) => (Some(false), 0, 0, Some(e)),
        }
    };
    ClusterStatus {
        dot: dot_for(reachable, critical, warning),
        context,
        cluster,
        current,
        reachable,
        critical,
        warning,
        detail: current,
        error,
        skipped: false,
    }
}

/// Sweeps over every context, at most 4 at a time, one sweep at a time;
/// pushes the result. A request while one runs makes it run once more.
pub async fn poll_once(app: &AppHandle) {
    let s = state(app);
    if !begin_sweep(&s.sweeping, &s.rerun) {
        return;
    }
    loop {
        s.rerun.store(false, Ordering::SeqCst);
        sweep(app).await;
        if !s.rerun.load(Ordering::SeqCst) {
            break;
        }
    }
    s.sweeping.store(false, Ordering::SeqCst);
}

async fn sweep(app: &AppHandle) {
    let Ok(contexts) = kubernetes::list_contexts().await else {
        return;
    };
    let current = kubernetes::get_current_context_name().await.ok();
    let cfg = read_kubeconfig();
    let limit = std::sync::Arc::new(tokio::sync::Semaphore::new(4));
    let mut set = tokio::task::JoinSet::new();
    for c in contexts {
        let is_current = current.as_deref() == Some(c.name.as_str());
        let plugin = cfg.as_ref().is_some_and(|k| uses_plugin_auth(k, &c.name));
        let (app, limit) = (app.clone(), limit.clone());
        set.spawn(async move {
            let _permit = limit.acquire_owned().await;
            check_one(app, c.name, c.cluster, is_current, plugin).await
        });
    }
    let mut clusters = Vec::new();
    while let Some(r) = set.join_next().await {
        if let Ok(s) = r {
            clusters.push(s);
        }
    }
    // Current first, then problems, then by name.
    clusters.sort_by(|a, b| {
        b.current
            .cmp(&a.current)
            .then(b.dot.cmp(&a.dot))
            .then(a.context.cmp(&b.context))
    });
    {
        let s = state(app);
        let mut snap = s.snapshot.lock().unwrap_or_else(|e| e.into_inner());
        snap.clusters = clusters;
        snap.checked_at = Some(chrono::Utc::now().to_rfc3339());
    }
    refresh_icon(app);
    let _ = app.emit(TRAY_STATUS, snapshot(app));
}

fn start_poller(app: &AppHandle) {
    let s = state(app);
    let epoch = s.epoch.fetch_add(1, Ordering::SeqCst) + 1;
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            let st = app.state::<TrayState>();
            if !st.enabled.load(Ordering::SeqCst) || st.epoch.load(Ordering::SeqCst) != epoch {
                return;
            }
            poll_once(&app).await;
            tokio::time::sleep(POLL_EVERY).await;
        }
    });
}

/// A context switch changes which cluster gets the full summary: re-poll now.
pub fn on_context_changed(app: &AppHandle) {
    if state(app).enabled.load(Ordering::SeqCst) {
        let app = app.clone();
        tauri::async_runtime::spawn(async move { poll_once(&app).await });
    }
}

// ============ Icon + popover ============

fn refresh_icon(app: &AppHandle) {
    let snap = snapshot(app);
    let dot = worst(&snap.clusters);
    let (rgba, template) = draw_icon(dot);
    let s = state(app);
    let icon = s.icon.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if let Some(icon) = icon {
        let _ = icon.set_icon(Some(Image::new_owned(rgba, ICON, ICON)));
        let _ = icon.set_icon_as_template(template);
        let bad = snap
            .clusters
            .iter()
            .filter(|c| matches!(c.dot, Dot::Red | Dot::Amber))
            .count();
        let tip = match bad {
            0 => "Apex, all clusters healthy".to_string(),
            1 => "Apex, 1 cluster needs attention".to_string(),
            n => format!("Apex, {n} clusters need attention"),
        };
        let _ = icon.set_tooltip(Some(tip));
    }
}

fn popover(app: &AppHandle) -> Result<tauri::WebviewWindow> {
    if let Some(w) = app.get_webview_window(POPOVER) {
        return Ok(w);
    }
    WebviewWindowBuilder::new(app, POPOVER, WebviewUrl::App("tray.html".into()))
        .title("Apex")
        .inner_size(320.0, 420.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()
        .map_err(|e| AppError::Custom(format!("Failed to create menu bar popover: {e}")))
}

fn toggle_popover(app: &AppHandle, anchor: Option<tauri::Rect>) {
    let Ok(w) = popover(app) else { return };
    if w.is_visible().unwrap_or(false) {
        let _ = w.hide();
        return;
    }
    if let Some(rect) = anchor {
        if let Some((x, y)) = popover_position(&w, rect) {
            let _ = w.set_position(PhysicalPosition::new(x, y));
        }
    }
    let _ = w.show();
    let _ = w.set_focus();
}

/// Under the icon, centred, clamped inside the monitor the icon is on (which
/// can sit at negative x left of the primary display), in its scale.
fn popover_position(w: &tauri::WebviewWindow, rect: tauri::Rect) -> Option<(f64, f64)> {
    let guess = w.scale_factor().unwrap_or(2.0);
    let pos = rect.position.to_physical::<f64>(guess);
    let mon = w
        .monitor_from_point(pos.x, pos.y)
        .ok()
        .flatten()
        .or_else(|| w.primary_monitor().ok().flatten())?;
    let scale = mon.scale_factor();
    let (pos, size) = (
        rect.position.to_physical::<f64>(scale),
        rect.size.to_physical::<f64>(scale),
    );
    let width = 320.0 * scale;
    let (mx, mw) = (mon.position().x as f64, mon.size().width as f64);
    Some(clamp_popover_x(
        pos.x + size.width / 2.0 - width / 2.0,
        width,
        mx,
        mw,
        8.0 * scale,
    ))
    .map(|x| (x, pos.y + size.height + 4.0 * scale))
}

/// Keep `width` inside `[mon_x, mon_x + mon_w]` with a margin (pure, unit-tested).
pub fn clamp_popover_x(x: f64, width: f64, mon_x: f64, mon_w: f64, margin: f64) -> f64 {
    let lo = mon_x + margin;
    let hi = mon_x + mon_w - width - margin;
    if hi < lo {
        lo
    } else {
        x.clamp(lo, hi)
    }
}

/// Popover closes when it loses focus, like a menu (wired in lib.rs).
pub fn on_popover_blur(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(POPOVER) {
        let _ = w.hide();
    }
}

pub fn hide_popover(app: &AppHandle) {
    on_popover_blur(app);
}

fn build_icon(app: &AppHandle) -> Result<TrayIcon> {
    let (rgba, template) = draw_icon(Dot::Grey);
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::new_owned(rgba, ICON, ICON))
        .icon_as_template(template)
        .tooltip("Apex")
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                toggle_popover(tray.app_handle(), Some(rect));
            }
        })
        .build(app)
        .map_err(|e| AppError::Custom(format!("Failed to create menu bar icon: {e}")))
}

/// Turn the menu bar item on or off; persists the choice once it worked.
pub fn set_enabled(app: &AppHandle, enabled: bool) -> Result<()> {
    let s = state(app);
    let was = s.enabled.load(Ordering::SeqCst);
    if enabled && !was {
        let icon = build_icon(app)?;
        *s.icon.lock().unwrap_or_else(|e| e.into_inner()) = Some(icon);
        s.enabled.store(true, Ordering::SeqCst);
        start_poller(app);
    } else if !enabled && was {
        s.enabled.store(false, Ordering::SeqCst);
        s.epoch.fetch_add(1, Ordering::SeqCst); // stops the poll loop
        if let Some(icon) = s.icon.lock().unwrap_or_else(|e| e.into_inner()).take() {
            let _ = icon.set_visible(false);
        }
        let _ = app.remove_tray_by_id(TRAY_ID);
        s.clients.lock().unwrap_or_else(|e| e.into_inner()).clear();
        // Close (not hide): a hidden window would keep a windowless app alive.
        if let Some(w) = app.get_webview_window(POPOVER) {
            let _ = w.close();
        }
    }
    save_enabled(app, enabled);
    let _ = app.emit(TRAY_STATUS, snapshot(app));
    crate::shell::sync_tray_menu(app, enabled);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(status: Status, severity: Option<Severity>) -> Verdict {
        Verdict {
            status,
            reason: String::new(),
            rule_id: "t".into(),
            since: None,
            severity,
            tone: None,
        }
    }

    #[test]
    fn critical_is_red_everything_else_amber() {
        let vs = [
            v(Status::Ok, None),
            v(Status::Fail, Some(Severity::Critical)),
            v(Status::Fail, Some(Severity::Error)),
            v(Status::Warn, Some(Severity::Warning)),
            v(Status::Unknown, None),
        ];
        assert_eq!(count_issues(&vs), (1, 2));
    }

    #[test]
    fn dot_reduction() {
        assert_eq!(dot_for(None, 0, 0), Dot::Grey);
        assert_eq!(dot_for(Some(false), 0, 0), Dot::Red);
        assert_eq!(dot_for(Some(true), 1, 5), Dot::Red);
        assert_eq!(dot_for(Some(true), 0, 1), Dot::Amber);
        assert_eq!(dot_for(Some(true), 0, 0), Dot::Green);
    }

    fn status(dot: Dot) -> ClusterStatus {
        ClusterStatus {
            context: "c".into(),
            cluster: "c".into(),
            current: false,
            dot,
            reachable: None,
            critical: 0,
            warning: 0,
            detail: false,
            error: None,
            skipped: false,
        }
    }

    #[test]
    fn one_sweep_at_a_time_with_one_rerun() {
        let (sw, rr) = (AtomicBool::new(false), AtomicBool::new(false));
        assert!(begin_sweep(&sw, &rr));
        assert!(
            !begin_sweep(&sw, &rr),
            "second caller doesn't start a parallel sweep"
        );
        assert!(
            rr.load(Ordering::SeqCst),
            "it asks the running one to go again"
        );
    }

    #[test]
    fn plugin_auth_contexts_are_detected() {
        let cfg: kube::config::Kubeconfig = serde_yaml::from_str(
            r#"
apiVersion: v1
kind: Config
contexts:
- name: eks
  context: { cluster: eks, user: aws }
- name: kind
  context: { cluster: kind, user: kind-admin }
- name: oidc
  context: { cluster: c, user: oidc-user }
users:
- name: aws
  user:
    exec: { apiVersion: client.authentication.k8s.io/v1beta1, command: aws, args: [eks, get-token] }
- name: kind-admin
  user: { client-certificate-data: abc, client-key-data: def }
- name: oidc-user
  user:
    auth-provider: { name: oidc, config: {} }
"#,
        )
        .unwrap();
        assert!(uses_plugin_auth(&cfg, "eks"));
        assert!(uses_plugin_auth(&cfg, "oidc"));
        assert!(!uses_plugin_auth(&cfg, "kind"));
        assert!(!uses_plugin_auth(&cfg, "missing"));
    }

    #[test]
    fn popover_stays_on_its_monitor() {
        // Centred when there's room.
        assert_eq!(clamp_popover_x(500.0, 640.0, 0.0, 3000.0, 16.0), 500.0);
        // Pushed in from the right edge.
        assert_eq!(
            clamp_popover_x(2600.0, 640.0, 0.0, 3000.0, 16.0),
            3000.0 - 640.0 - 16.0
        );
        // A display left of the primary one (negative x) keeps negative coordinates.
        assert_eq!(
            clamp_popover_x(-900.0, 640.0, -2000.0, 2000.0, 16.0),
            -900.0
        );
        assert_eq!(
            clamp_popover_x(-100.0, 640.0, -2000.0, 2000.0, 16.0),
            -656.0
        );
    }

    #[test]
    fn worst_of_clusters() {
        assert_eq!(worst(&[]), Dot::Grey);
        assert_eq!(worst(&[status(Dot::Green), status(Dot::Grey)]), Dot::Green);
        assert_eq!(worst(&[status(Dot::Green), status(Dot::Amber)]), Dot::Amber);
        assert_eq!(
            worst(&[status(Dot::Amber), status(Dot::Red), status(Dot::Green)]),
            Dot::Red
        );
    }

    #[test]
    fn icon_is_a_template_until_something_is_wrong() {
        let (px, template) = draw_icon(Dot::Green);
        assert!(template);
        assert_eq!(px.len(), (ICON * ICON * 4) as usize);
        let at = |px: &[u8], x: usize, y: usize| px[(y * ICON as usize + x) * 4..][..4].to_vec();
        assert_eq!(
            at(&px, 34, 34)[3],
            255,
            "triangle covers the corner without a dot"
        );
        let (px, template) = draw_icon(Dot::Red);
        assert!(!template);
        assert_eq!(at(&px, 34, 34), vec![229, 72, 77, 255], "red dot");
        assert_eq!(at(&px, 22, 3)[3], 0, "transparent above the apex");
    }
}
