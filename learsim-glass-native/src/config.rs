//! Persistent configuration — identical shape to the Tauri client so the admin
//! app talks to both interchangeably.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const DEFAULT_PORT: u16 = 8770;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub monitor: Option<usize>,
    pub view_id: String,
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

    /// A settings value as a trimmed, non-empty string.
    pub fn setting_str(&self, key: &str) -> Option<&str> {
        self.settings
            .get(key)
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }

    /// A settings value as a number (accepts JSON number or numeric string).
    pub fn setting_u64(&self, key: &str) -> Option<u64> {
        match self.settings.get(key) {
            Some(Value::Number(n)) => n.as_u64(),
            Some(Value::String(s)) => s.trim().parse().ok(),
            _ => None,
        }
    }

    pub fn setting_bool(&self, key: &str) -> bool {
        matches!(self.settings.get(key), Some(Value::Bool(true)))
            || matches!(self.settings.get(key), Some(Value::String(s)) if s == "true")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub device: DeviceInfo,
    pub port: u16,
    pub screens: Vec<ScreenConfig>,
}

impl AppConfig {
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
            screens: vec![ScreenConfig::new("screen-1", "Screen 1", Some(0))],
        }
    }

    pub fn screen(&self, id: &str) -> Option<&ScreenConfig> {
        self.screens.iter().find(|s| s.id == id)
    }

    pub fn screen_index(&self, id: &str) -> Option<usize> {
        self.screens.iter().position(|s| s.id == id)
    }
}

/// Default config path: `~/.config/learsim-glass/config.json`.
pub fn default_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("learsim-glass")
        .join("config.json")
}

pub fn load(path: &Path) -> AppConfig {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|err| {
            eprintln!("[config] {} is invalid ({err}); using defaults", path.display());
            AppConfig::default_for_host()
        }),
        Err(_) => AppConfig::default_for_host(),
    }
}

pub fn save(path: &Path, config: &AppConfig) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(config)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, text)
}
