//! Shared, thread-safe application state.

use crate::config::{AppConfig, ConfigPath};
use std::sync::Mutex;

/// Managed by Tauri (`app.manage(Arc::new(AppState { .. }))`) and shared with
/// the control server. The `Mutex` is a plain `std` mutex; guards are never
/// held across an `.await` point, so it is safe inside async handlers.
pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub config_path: ConfigPath,
    /// The resolved URL of our own app document (e.g. `tauri://localhost/` in
    /// production, `http://localhost:1420/` in dev). Captured at startup so a
    /// glass screen can navigate *back* to a local view after having been
    /// pointed at an external glassout engine URL.
    pub app_url: Mutex<Option<String>>,
    /// Shared HTTP client used to probe glassout engine health (`/status`).
    pub http: reqwest::Client,
}

impl AppState {
    pub fn new(config: AppConfig, config_path: ConfigPath, http: reqwest::Client) -> Self {
        Self {
            config: Mutex::new(config),
            config_path,
            app_url: Mutex::new(None),
            http,
        }
    }

    /// Persist the current config to disk. Logs on failure rather than
    /// panicking — a failed write should never take down a display.
    pub fn persist(&self) {
        let cfg = self.config.lock().unwrap();
        if let Err(err) = crate::config::save(&self.config_path, &cfg) {
            eprintln!("[state] failed to persist config: {err}");
        }
    }
}
