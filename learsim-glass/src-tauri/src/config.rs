//! Persistent configuration for a glass device.
//!
//! A *device* is one machine running `learsim-glass` (e.g. a Raspberry Pi).
//! A device has one or more *screens* — normally one per physical monitor.
//! Each screen shows exactly one *view*. Two monitors on the same Pi therefore
//! produce two independent [`ScreenConfig`] entries, each with its own view and
//! settings — which is the whole point of the per-screen model.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Default TCP port for the embedded control server.
///
/// Deliberately NOT 8787 — that port belongs to the glassout engine. Keeping
/// them distinct lets a glass display and a glassout engine run on the same
/// machine without colliding.
pub const DEFAULT_PORT: u16 = 8770;

/// Identity of the device this instance runs on.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    /// Stable, randomly generated id (persists across restarts).
    pub id: String,
    /// Human-friendly name — defaults to the machine hostname.
    pub name: String,
}

/// Configuration for a single screen (one physical monitor).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenConfig {
    /// Stable id; also used as the Tauri window label.
    pub id: String,
    /// Human-friendly name shown in the admin app.
    pub name: String,
    /// Index into the device's monitor list this screen is pinned to.
    #[serde(default)]
    pub monitor: Option<usize>,
    /// Which view is currently displayed on this screen.
    pub view_id: String,
    /// View-specific settings (schema is defined by the view — see `views.rs`).
    #[serde(default)]
    pub settings: Map<String, Value>,
}

impl ScreenConfig {
    pub fn new(id: impl Into<String>, name: impl Into<String>, monitor: Option<usize>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            monitor,
            view_id: crate::views::DEFAULT_VIEW_ID.to_string(),
            settings: Map::new(),
        }
    }
}

/// The full on-disk configuration document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub device: DeviceInfo,
    /// Port the control server listens on (bound on `0.0.0.0`).
    pub port: u16,
    pub screens: Vec<ScreenConfig>,
}

impl AppConfig {
    /// Build a fresh default config for a first run.
    pub fn default_for_host() -> Self {
        let name = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "learsim-glass".to_string());
        Self {
            device: DeviceInfo {
                id: uuid::Uuid::new_v4().to_string(),
                name,
            },
            port: DEFAULT_PORT,
            screens: Vec::new(),
        }
    }

    pub fn screen(&self, id: &str) -> Option<&ScreenConfig> {
        self.screens.iter().find(|s| s.id == id)
    }

    pub fn screen_index(&self, id: &str) -> Option<usize> {
        self.screens.iter().position(|s| s.id == id)
    }
}

/// A flattened view of a screen sent to the frontend / admin, including the
/// owning device name so a window can render a self-describing standby screen.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenState {
    pub id: String,
    pub name: String,
    pub device_name: String,
    pub view_id: String,
    pub settings: Map<String, Value>,
}

impl ScreenState {
    pub fn from_screen(screen: &ScreenConfig, device_name: &str) -> Self {
        Self {
            id: screen.id.clone(),
            name: screen.name.clone(),
            device_name: device_name.to_string(),
            view_id: screen.view_id.clone(),
            settings: screen.settings.clone(),
        }
    }
}

/// Load the config from `path`. Returns `(config, existed)` where `existed`
/// is false when a fresh default was created (used to decide whether to
/// auto-provision one screen per detected monitor).
pub fn load(path: &Path) -> (AppConfig, bool) {
    match fs::read_to_string(path) {
        Ok(text) => match serde_json::from_str::<AppConfig>(&text) {
            Ok(cfg) => (cfg, true),
            Err(err) => {
                eprintln!(
                    "[config] {} is invalid ({err}); starting from defaults",
                    path.display()
                );
                (AppConfig::default_for_host(), false)
            }
        },
        Err(_) => (AppConfig::default_for_host(), false),
    }
}

/// Persist the config, creating the parent directory if necessary.
pub fn save(path: &Path, config: &AppConfig) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(config)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, text)
}

/// Convenience alias used across the crate.
pub type ConfigPath = PathBuf;
