//! Fullscreen kiosk window management.
//!
//! One decoration-less fullscreen window is created per screen. Each window is
//! labelled with its screen id, so the frontend can read `getCurrentWindow()`
//! to know which screen it represents. When a screen is pinned to a monitor
//! index, the window is positioned onto that monitor before going fullscreen —
//! this is how two monitors on one Pi end up showing two different views.

use crate::config::ScreenConfig;
use tauri::{AppHandle, Manager, PhysicalPosition, Url, WebviewUrl, WebviewWindowBuilder};

/// Create the kiosk window for a screen if it does not already exist.
pub fn create_screen_window(app: &AppHandle, screen: &ScreenConfig) -> tauri::Result<()> {
    if app.get_webview_window(&screen.id).is_some() {
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(app, &screen.id, WebviewUrl::App("index.html".into()))
        .title(format!("learsim-glass · {}", screen.name))
        .decorations(false)
        .resizable(false)
        .build()?;

    // Pin to the requested monitor, if any, before fullscreening.
    if let Some(monitor_idx) = screen.monitor {
        if let Ok(monitors) = window.available_monitors() {
            if let Some(monitor) = monitors.get(monitor_idx) {
                let pos = monitor.position();
                let _ = window.set_position(PhysicalPosition {
                    x: pos.x,
                    y: pos.y,
                });
            }
        }
    }

    // Kiosk: fullscreen with no cursor chrome. Failure is non-fatal.
    if let Err(err) = window.set_fullscreen(true) {
        eprintln!("[windows] could not fullscreen '{}': {err}", screen.id);
    }

    Ok(())
}

/// Close the kiosk window for a screen (used when a screen is deleted).
pub fn close_screen_window(app: &AppHandle, screen_id: &str) {
    if let Some(window) = app.get_webview_window(screen_id) {
        if let Err(err) = window.close() {
            eprintln!("[windows] could not close '{screen_id}': {err}");
        }
    }
}

/// Point a screen's window at the right content for its assigned view.
///
/// - A `glassout` screen navigates to the engine's viewer URL (top-level
///   navigation, so a LAN `http://` engine isn't blocked as mixed content).
/// - Any local view (standby, clock, …) navigates back to our own app document
///   if the window is currently showing an external URL; otherwise it does
///   nothing and lets the live `screen-changed` event re-render in place.
///
/// `app_url` is the resolved URL of our app document (from `AppState::app_url`).
/// Must be called on the main thread.
pub fn apply_screen_view(app: &AppHandle, screen: &ScreenConfig, app_url: Option<String>) {
    let Some(window) = app.get_webview_window(&screen.id) else {
        return;
    };
    let current = window.url().ok();

    if screen.view_id == "glassout" {
        let Some(target) = crate::glassout::build_view_url(&screen.settings) else {
            // Incomplete glassout settings → fall back to the local app doc.
            navigate_to_app(&window, current.as_ref(), app_url);
            return;
        };
        if let Ok(url) = Url::parse(&target) {
            if current.as_ref().map(Url::as_str) != Some(url.as_str()) {
                if let Err(err) = window.navigate(url) {
                    eprintln!("[windows] navigate '{}' failed: {err}", screen.id);
                }
            }
        }
    } else {
        navigate_to_app(&window, current.as_ref(), app_url);
    }
}

fn navigate_to_app(
    window: &tauri::WebviewWindow,
    current: Option<&Url>,
    app_url: Option<String>,
) {
    // Already on our app document → nothing to do (the SPA updates via event).
    if current.map(is_app_url).unwrap_or(false) {
        return;
    }
    if let Some(url_str) = app_url {
        if let Ok(url) = Url::parse(&url_str) {
            let _ = window.navigate(url);
        }
    }
}

/// Whether a URL points at our own bundled app document rather than an external
/// (glassout engine) page.
fn is_app_url(url: &Url) -> bool {
    if url.scheme() == "tauri" {
        return true;
    }
    matches!(
        url.host_str(),
        Some("localhost") | Some("127.0.0.1") | Some("tauri.localhost")
    )
}

/// Number of monitors visible to the app. Requires an existing window to query
/// from (Tauri exposes monitor enumeration per-window). Returns `None` if it
/// cannot be determined.
pub fn monitor_count(app: &AppHandle, from_window: &str) -> Option<usize> {
    let window = app.get_webview_window(from_window)?;
    window.available_monitors().ok().map(|m| m.len())
}
