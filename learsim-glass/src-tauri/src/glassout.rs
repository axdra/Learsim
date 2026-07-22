//! glassout integration for the display side.
//!
//! glassout (https://glassout.flyingart.dev) is an MSFS panel-capture engine.
//! It exposes an HTML viewer for every panel/composite it serves, e.g.
//! `http://<engine>:8787/panel/PFD_Captain?fps=60&fit=stretch`. Per the SDK
//! docs, a browser client should simply load these viewer URLs directly — the
//! engine handles frame decode, adaptive quality, and click forwarding.
//!
//! So on a glass screen assigned the `glassout` view, we navigate the kiosk
//! window straight at the engine's viewer URL (see `windows::apply_screen_view`).
//! This module is the single place that knows how to turn a screen's settings
//! into that URL. It mirrors the SDK's `buildPanelUrl` semantics.
//!
//! Selecting *which* panel is a job for the admin app, which uses the
//! `glassout-client` SDK (LAN discovery + `panels`/`configs` events) to present
//! a real list. Here we only need the resulting engine URL + panel id.

use serde_json::{Map, Value};

fn setting_str<'a>(settings: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    settings
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn setting_number(settings: &Map<String, Value>, key: &str) -> Option<u64> {
    match settings.get(key) {
        Some(Value::Number(n)) => n.as_u64(),
        Some(Value::String(s)) => s.trim().parse().ok(),
        _ => None,
    }
}

fn setting_bool(settings: &Map<String, Value>, key: &str) -> bool {
    matches!(settings.get(key), Some(Value::Bool(true)))
        || matches!(settings.get(key), Some(Value::String(s)) if s == "true")
}

/// Build the glassout viewer URL a screen should navigate to, from its
/// free-form settings. Returns `None` if the settings are incomplete (in which
/// case the screen falls back to standby).
///
/// Recognised settings:
///   engineUrl  — base URL of the engine, e.g. "http://192.168.1.42:8787"
///   panelId    — panel to show, e.g. "PFD_Captain"
///   fit        — "contain" (default, omitted) | "stretch" | "native"
///   targetFps  — per-viewer fps cap, 10–120
///   debug      — enable the engine's latency overlay
///   path       — advanced: a full viewer path (e.g. "/canvas?..." or
///                "/instance/...") that overrides the panel builder verbatim.
pub fn build_view_url(settings: &Map<String, Value>) -> Option<String> {
    let base = setting_str(settings, "engineUrl")?.trim_end_matches('/');

    // Advanced override: caller supplied a ready-made viewer path.
    if let Some(path) = setting_str(settings, "path") {
        let sep = if path.starts_with('/') { "" } else { "/" };
        return Some(format!("{base}{sep}{path}"));
    }

    let panel = setting_str(settings, "panelId")?;
    let panel_enc = encode_segment(panel);

    let mut query: Vec<String> = Vec::new();
    if let Some(fps) = setting_number(settings, "targetFps") {
        query.push(format!("fps={fps}"));
    }
    if let Some(fit) = setting_str(settings, "fit") {
        // "contain" is the engine default and omitted for compatibility.
        if fit != "contain" {
            query.push(format!("fit={fit}"));
        }
    }
    if setting_bool(settings, "debug") {
        query.push("debug=1".to_string());
    }

    let mut url = format!("{base}/panel/{panel_enc}");
    if !query.is_empty() {
        url.push('?');
        url.push_str(&query.join("&"));
    }
    Some(url)
}

/// Minimal path-segment encoding — panel ids are normally already URL-safe
/// (e.g. "PFD_Captain"); this just guards against spaces and a few specials.
fn encode_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn map(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    #[test]
    fn builds_panel_url() {
        let s = map(json!({
            "engineUrl": "http://192.168.1.42:8787/",
            "panelId": "PFD_Captain",
            "fit": "stretch",
            "targetFps": 60,
            "debug": true
        }));
        assert_eq!(
            build_view_url(&s).unwrap(),
            "http://192.168.1.42:8787/panel/PFD_Captain?fps=60&fit=stretch&debug=1"
        );
    }

    #[test]
    fn contain_is_omitted() {
        let s = map(json!({
            "engineUrl": "http://host:8787",
            "panelId": "MFD",
            "fit": "contain"
        }));
        assert_eq!(build_view_url(&s).unwrap(), "http://host:8787/panel/MFD");
    }

    #[test]
    fn path_override_wins() {
        let s = map(json!({
            "engineUrl": "http://host:8787",
            "panelId": "ignored",
            "path": "/canvas?entries=abc"
        }));
        assert_eq!(
            build_view_url(&s).unwrap(),
            "http://host:8787/canvas?entries=abc"
        );
    }

    #[test]
    fn incomplete_returns_none() {
        let s = map(json!({ "engineUrl": "http://host:8787" }));
        assert!(build_view_url(&s).is_none());
    }
}
