//! Shared application state (config + change notification).

use crate::config::{self, AppConfig};
use std::path::PathBuf;
use std::sync::Mutex;
use tokio::sync::watch;

pub struct Shared {
    pub config: Mutex<AppConfig>,
    pub config_path: PathBuf,
    /// Bumped whenever the config changes so the glassout supervisor re-evaluates.
    pub change_tx: watch::Sender<u64>,
}

impl Shared {
    pub fn new(config: AppConfig, config_path: PathBuf) -> Self {
        let (change_tx, _) = watch::channel(0u64);
        Self {
            config: Mutex::new(config),
            config_path,
            change_tx,
        }
    }

    /// Notify watchers that the config changed.
    pub fn bump(&self) {
        let next = self.change_tx.borrow().wrapping_add(1);
        let _ = self.change_tx.send(next);
    }

    /// Persist the current config to disk (logs on failure).
    pub fn persist(&self) {
        let cfg = self.config.lock().unwrap();
        if let Err(err) = config::save(&self.config_path, &cfg) {
            eprintln!("[state] failed to persist config: {err}");
        }
    }
}
