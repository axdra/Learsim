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

/// Navigate a window to `target` unless it is already there. Used to point a
/// glassout screen at its engine viewer URL. Must run on the main thread.
pub(crate) fn navigate_if_changed(window: &tauri::WebviewWindow, target: &Url) {
    let current = window.url().ok();
    if current.as_ref().map(Url::as_str) != Some(target.as_str()) {
        if let Err(err) = window.navigate(target.clone()) {
            eprintln!("[windows] navigate '{}' failed: {err}", window.label());
        }
    }
}

/// Ensure a window is showing our own app document (which renders the assigned
/// local view, or the glassout "connecting…" placeholder). If it already is,
/// do nothing so the live `screen-changed` event can re-render in place without
/// a reload. Must run on the main thread.
pub(crate) fn ensure_on_app(window: &tauri::WebviewWindow, app_url: &Url) {
    let on_app = window.url().ok().as_ref().map(is_app_url).unwrap_or(false);
    if on_app {
        return;
    }
    let _ = window.navigate(app_url.clone());
}

/// Whether a URL points at our own bundled app document rather than an external
/// (glassout engine) page.
pub(crate) fn is_app_url(url: &Url) -> bool {
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
