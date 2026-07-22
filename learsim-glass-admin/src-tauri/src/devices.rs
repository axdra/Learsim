//! Persistent list of known glass devices (host + control-server port).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StoredDevice {
    pub host: String,
    pub port: u16,
    /// Last-known friendly name (cached from the device's `/api/device`).
    #[serde(default)]
    pub label: Option<String>,
}

impl StoredDevice {
    fn same_endpoint(&self, host: &str, port: u16) -> bool {
        self.host == host && self.port == port
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct DeviceFile {
    devices: Vec<StoredDevice>,
}

/// Thread-safe store persisted to a JSON file.
pub struct DeviceStore {
    path: PathBuf,
    devices: Mutex<Vec<StoredDevice>>,
}

impl DeviceStore {
    pub fn load(path: PathBuf) -> Self {
        let devices = fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str::<DeviceFile>(&t).ok())
            .map(|f| f.devices)
            .unwrap_or_default();
        Self {
            path,
            devices: Mutex::new(devices),
        }
    }

    pub fn list(&self) -> Vec<StoredDevice> {
        self.devices.lock().unwrap().clone()
    }

    /// Insert or update a device by endpoint, then persist.
    pub fn upsert(&self, host: String, port: u16, label: Option<String>) {
        {
            let mut devices = self.devices.lock().unwrap();
            match devices.iter_mut().find(|d| d.same_endpoint(&host, port)) {
                Some(existing) => {
                    if label.is_some() {
                        existing.label = label;
                    }
                }
                None => devices.push(StoredDevice { host, port, label }),
            }
        }
        self.persist();
    }

    pub fn remove(&self, host: &str, port: u16) {
        self.devices
            .lock()
            .unwrap()
            .retain(|d| !d.same_endpoint(host, port));
        self.persist();
    }

    fn persist(&self) {
        let file = DeviceFile {
            devices: self.devices.lock().unwrap().clone(),
        };
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(&file) {
            Ok(text) => {
                if let Err(err) = fs::write(&self.path, text) {
                    eprintln!("[devices] persist failed: {err}");
                }
            }
            Err(err) => eprintln!("[devices] serialize failed: {err}"),
        }
    }
}

/// Path helper: the store lives next to other app config.
pub fn store_path(dir: &Path) -> PathBuf {
    dir.join("devices.json")
}
