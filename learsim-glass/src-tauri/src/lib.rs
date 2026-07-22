//! learsim-glass — kiosk display for glassout views.
//!
//! Boot sequence (see [`run`]):
//!   1. Load (or create) the device config.
//!   2. Create one decoration-less fullscreen window per configured screen,
//!      each labelled with its screen id and pinned to its monitor.
//!   3. On first run, auto-provision one screen per detected monitor — so a Pi
//!      with two monitors gets two independent screen configs out of the box.
//!   4. Navigate each window to its assigned view (glassout engine URL for
//!      glassout screens; the local app document otherwise).
//!   5. Start the control server so the admin app can reassign views live.

mod commands;
mod config;
mod glassout;
mod reconcile;
mod server;
mod state;
mod views;
mod windows;

use state::AppState;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::get_screen_state])
        .setup(|app| {
            let handle = app.handle().clone();

            let config_path = handle
                .path()
                .app_config_dir()
                .map(|dir| dir.join("config.json"))
                .unwrap_or_else(|_| std::path::PathBuf::from("learsim-glass.config.json"));

            let (mut cfg, existed) = config::load(&config_path);
            if cfg.screens.is_empty() {
                cfg.screens
                    .push(config::ScreenConfig::new("screen-1", "Screen 1", Some(0)));
            }

            // Create the first window (loads our app document). The app-document
            // URL is captured lazily via the window's on_page_load handler (see
            // windows.rs) — reading it synchronously here would return the
            // transient "about:blank" and navigating to that blanks the window.
            windows::create_screen_window(&handle, &cfg.screens[0])?;

            // First run: provision one screen per additional monitor.
            if !existed {
                if let Some(monitors) = windows::monitor_count(&handle, &cfg.screens[0].id) {
                    for i in cfg.screens.len()..monitors {
                        let id = format!("screen-{}", i + 1);
                        let name = format!("Screen {}", i + 1);
                        cfg.screens
                            .push(config::ScreenConfig::new(id, name, Some(i)));
                    }
                }
            }

            // Create windows for the remaining screens.
            for screen in cfg.screens.iter().skip(1) {
                windows::create_screen_window(&handle, screen)?;
            }

            let port = cfg.port;
            let screens_snapshot = cfg.screens.clone();

            // Shared HTTP client for probing glassout engine health.
            let http = reqwest::Client::builder().build().unwrap_or_default();

            // Publish shared state and persist any first-run provisioning.
            let app_state = Arc::new(AppState::new(cfg, config_path.clone(), http.clone()));
            {
                let guard = app_state.config.lock().unwrap();
                let _ = config::save(&config_path, &*guard);
            }
            app.manage(app_state.clone());

            // In debug builds, open devtools so a window can be inspected.
            #[cfg(debug_assertions)]
            for screen in &screens_snapshot {
                if let Some(w) = handle.get_webview_window(&screen.id) {
                    w.open_devtools();
                }
            }

            // Point each window at its assigned view (self-heals glassout).
            // `app_url` may still be None here (captured on first page load);
            // that's fine — reconcile never navigates to an unknown URL, so the
            // window completes its own load to the app document.
            for screen in &screens_snapshot {
                let app_url = app_state.app_url.lock().unwrap().clone();
                reconcile::kick(&handle, http.clone(), screen.clone(), app_url);
            }

            // Keep glassout screens healed as their engines come and go.
            reconcile::spawn_monitor(handle.clone(), app_state.clone());

            // Start the LAN control server.
            let ctx = Arc::new(server::ServerCtx {
                state: app_state.clone(),
                app: handle.clone(),
            });
            tauri::async_runtime::spawn(async move {
                server::run(ctx, port).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running learsim-glass");
}
