//! Keeps each screen's window pointed at the right content — and self-heals
//! glassout screens as their engine comes and goes.
//!
//! Desired content per screen:
//!   - local view (standby/clock/…) → our app document (the SPA renders it)
//!   - glassout + engine reachable  → the engine's panel viewer URL
//!   - glassout + engine down / unset → our app document, where the SPA shows a
//!     branded "connecting to glassout…" placeholder
//!
//! A one-off [`kick`] runs on startup and after every admin change for an
//! instant response; a background [`spawn_monitor`] loop re-checks glassout
//! screens every few seconds so a rebooted sim PC recovers automatically
//! instead of leaving a browser error page on the panel.

use crate::config::ScreenConfig;
use crate::state::AppState;
use crate::{glassout, windows};
use serde_json::{Map, Value};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager, Url};

const MONITOR_INTERVAL: Duration = Duration::from_secs(5);
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Probe a glassout engine's `/status`. `true` when it responds successfully.
async fn engine_reachable(http: &reqwest::Client, settings: &Map<String, Value>) -> bool {
    let base = settings
        .get("engineUrl")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let Some(base) = base else {
        return false;
    };
    let url = format!("{}/status", base.trim_end_matches('/'));
    matches!(
        http.get(url).timeout(PROBE_TIMEOUT).send().await,
        Ok(resp) if resp.status().is_success()
    )
}

/// The engine URL a screen's window should show, or `None` for "our app
/// document" (a local view, or the glassout placeholder when the engine is
/// unreachable / unconfigured).
async fn desired_engine_url(http: &reqwest::Client, screen: &ScreenConfig) -> Option<String> {
    if screen.view_id != "glassout" {
        return None;
    }
    match glassout::build_view_url(&screen.settings) {
        Some(panel) if engine_reachable(http, &screen.settings).await => Some(panel),
        _ => None,
    }
}

/// Reconcile one screen's window a single time.
pub async fn reconcile_screen(
    app: AppHandle,
    http: reqwest::Client,
    screen: ScreenConfig,
    app_url: Option<String>,
) {
    let target = desired_engine_url(&http, &screen).await;
    let screen_id = screen.id;
    let app_for_closure = app.clone();
    let _ = app.run_on_main_thread(move || {
        let Some(window) = app_for_closure.get_webview_window(&screen_id) else {
            return;
        };
        match target {
            Some(engine_url) => {
                if let Ok(url) = Url::parse(&engine_url) {
                    windows::navigate_if_changed(&window, &url);
                }
            }
            None => {
                if let Some(url) = app_url.as_deref().and_then(|s| Url::parse(s).ok()) {
                    windows::ensure_on_app(&window, &url);
                }
            }
        }
    });
}

/// Fire a one-off reconcile for a screen (startup / after an admin change) so
/// the window responds immediately without waiting for the monitor loop.
pub fn kick(app: &AppHandle, http: reqwest::Client, screen: ScreenConfig, app_url: Option<String>) {
    let app = app.clone();
    tauri::async_runtime::spawn(reconcile_screen(app, http, screen, app_url));
}

/// Background loop that keeps glassout screens healed.
pub fn spawn_monitor(app: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(MONITOR_INTERVAL).await;

            // Snapshot the current glassout screens without holding the lock
            // across the awaits below.
            let (screens, app_url, http) = {
                let cfg = state.config.lock().unwrap();
                let screens: Vec<ScreenConfig> = cfg
                    .screens
                    .iter()
                    .filter(|s| s.view_id == "glassout")
                    .cloned()
                    .collect();
                (
                    screens,
                    state.app_url.lock().unwrap().clone(),
                    state.http.clone(),
                )
            };

            for screen in screens {
                reconcile_screen(app.clone(), http.clone(), screen, app_url.clone()).await;
            }
        }
    });
}
