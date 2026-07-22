//! Tauri commands: a thin HTTP proxy from the admin UI to glass devices'
//! control servers (and a reachability probe for glassout engines).
//!
//! Doing the HTTP from Rust (rather than `fetch` in the webview) sidesteps
//! CORS and mixed-content concerns and keeps a single place for timeouts.

use crate::devices::{store_path, DeviceStore, StoredDevice};
use serde_json::Value;
use std::time::Duration;
use tauri::{Manager, State};

/// Shared HTTP client + device store.
pub struct AdminState {
    pub http: reqwest::Client,
    pub store: DeviceStore,
}

impl AdminState {
    pub fn new(config_dir: std::path::PathBuf) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(6))
            .user_agent(concat!("learsim-glass-admin/", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_default();
        Self {
            http,
            store: DeviceStore::load(store_path(&config_dir)),
        }
    }
}

fn base_url(host: &str, port: u16) -> String {
    format!("http://{host}:{port}")
}

async fn get_json(state: &AdminState, url: &str) -> Result<Value, String> {
    let resp = state
        .http
        .get(url)
        .send()
        .await
        .map_err(|e| format!("request to {url} failed: {e}"))?;
    let status = resp.status();
    let body = resp
        .json::<Value>()
        .await
        .map_err(|e| format!("invalid JSON from {url}: {e}"))?;
    if !status.is_success() {
        return Err(format!("{url} returned HTTP {status}"));
    }
    Ok(body)
}

// --- device management ------------------------------------------------------

#[tauri::command]
pub fn list_devices(state: State<'_, AdminState>) -> Vec<StoredDevice> {
    state.store.list()
}

/// Validate an endpoint by fetching `/api/device`, cache it, and return the
/// device snapshot (device info + screens + view manifest).
#[tauri::command]
pub async fn add_device(
    host: String,
    port: u16,
    state: State<'_, AdminState>,
) -> Result<Value, String> {
    let snapshot = get_json(&state, &format!("{}/api/device", base_url(&host, port))).await?;
    let label = snapshot
        .get("device")
        .and_then(|d| d.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string());
    state.store.upsert(host, port, label);
    Ok(snapshot)
}

#[tauri::command]
pub fn remove_device(host: String, port: u16, state: State<'_, AdminState>) {
    state.store.remove(&host, port);
}

/// Lightweight liveness check for the sidebar status dots: `true` if the
/// device's control server answers `GET /api/health`.
#[tauri::command]
pub async fn ping_device(host: String, port: u16, state: State<'_, AdminState>) -> Result<bool, String> {
    let url = format!("{}/api/health", base_url(&host, port));
    Ok(matches!(
        state.http.get(&url).send().await,
        Ok(resp) if resp.status().is_success()
    ))
}

/// Re-fetch a device's live snapshot.
#[tauri::command]
pub async fn fetch_device(
    host: String,
    port: u16,
    state: State<'_, AdminState>,
) -> Result<Value, String> {
    let snapshot = get_json(&state, &format!("{}/api/device", base_url(&host, port))).await?;
    // Refresh the cached label opportunistically.
    if let Some(name) = snapshot
        .get("device")
        .and_then(|d| d.get("name"))
        .and_then(|n| n.as_str())
    {
        state.store.upsert(host, port, Some(name.to_string()));
    }
    Ok(snapshot)
}

// --- screen mutations -------------------------------------------------------

/// Update a screen (view id / settings / name). `body` is passed through to the
/// device's `PUT /api/screens/{id}`.
#[tauri::command]
pub async fn set_screen(
    host: String,
    port: u16,
    screen_id: String,
    body: Value,
    state: State<'_, AdminState>,
) -> Result<Value, String> {
    let url = format!("{}/api/screens/{}", base_url(&host, port), screen_id);
    let resp = state
        .http
        .put(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("PUT {url} failed: {e}"))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("{url} returned HTTP {status}: {text}"));
    }
    serde_json::from_str(&text).map_err(|e| format!("invalid JSON from {url}: {e}"))
}

#[tauri::command]
pub async fn add_screen(
    host: String,
    port: u16,
    body: Value,
    state: State<'_, AdminState>,
) -> Result<Value, String> {
    let url = format!("{}/api/screens", base_url(&host, port));
    let resp = state
        .http
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("POST {url} failed: {e}"))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("{url} returned HTTP {status}: {text}"));
    }
    serde_json::from_str(&text).map_err(|e| format!("invalid JSON from {url}: {e}"))
}

#[tauri::command]
pub async fn delete_screen(
    host: String,
    port: u16,
    screen_id: String,
    state: State<'_, AdminState>,
) -> Result<(), String> {
    let url = format!("{}/api/screens/{}", base_url(&host, port), screen_id);
    let resp = state
        .http
        .delete(&url)
        .send()
        .await
        .map_err(|e| format!("DELETE {url} failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("{url} returned HTTP {}", resp.status()));
    }
    Ok(())
}

// --- glassout engine probe --------------------------------------------------

/// Check that a glassout engine responds on `/status`. Used to validate the
/// `engineUrl` an operator enters for a glassout view. (Enumerating panels is
/// the job of the `glassout-client` SDK on the frontend — see the UI's
/// glassout module.)
#[tauri::command]
pub async fn probe_engine(engine_url: String, state: State<'_, AdminState>) -> Result<Value, String> {
    let url = format!("{}/status", engine_url.trim_end_matches('/'));
    get_json(&state, &url).await
}

/// Build the `AdminState` from Tauri's config dir.
pub fn build_state(app: &tauri::App) -> AdminState {
    let dir = app
        .handle()
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    AdminState::new(dir)
}
