//! View manifest — the catalogue of views this device can display.
//!
//! The manifest is the contract between the glass display and the admin app:
//! the admin fetches it from a device (`GET /api/device`) so its UI always
//! reflects exactly which views that device supports and what settings each
//! view accepts. The frontend view *renderers* (see `src/views/`) are keyed by
//! the same ids.
//!
//! Adding a view = add a descriptor here + a renderer in the frontend registry.

use serde::Serialize;
use serde_json::{json, Value};

/// The view shown before anything is configured.
pub const DEFAULT_VIEW_ID: &str = "standby";

/// A single configurable field of a view's settings.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewField {
    pub key: String,
    pub label: String,
    /// One of `"text"`, `"boolean"`, `"select"`.
    pub kind: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<ViewOption>,
    pub default: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct ViewOption {
    pub value: String,
    pub label: String,
}

/// Describes one view: its identity plus the settings it accepts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewDescriptor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub fields: Vec<ViewField>,
}

fn field_text(key: &str, label: &str, default: &str) -> ViewField {
    ViewField {
        key: key.into(),
        label: label.into(),
        kind: "text".into(),
        options: vec![],
        default: json!(default),
    }
}

fn field_bool(key: &str, label: &str, default: bool) -> ViewField {
    ViewField {
        key: key.into(),
        label: label.into(),
        kind: "boolean".into(),
        options: vec![],
        default: json!(default),
    }
}

fn field_select(key: &str, label: &str, options: &[(&str, &str)], default: &str) -> ViewField {
    ViewField {
        key: key.into(),
        label: label.into(),
        kind: "select".into(),
        options: options
            .iter()
            .map(|(v, l)| ViewOption {
                value: (*v).into(),
                label: (*l).into(),
            })
            .collect(),
        default: json!(default),
    }
}

/// The full catalogue of views this build supports.
pub fn view_manifest() -> Vec<ViewDescriptor> {
    vec![
        ViewDescriptor {
            id: "standby".into(),
            name: "Standby".into(),
            description: "Idle screen showing the device and screen name.".into(),
            fields: vec![field_text(
                "message",
                "Message",
                "",
            )],
        },
        ViewDescriptor {
            id: "clock".into(),
            name: "Clock".into(),
            description: "Large clock with optional seconds and date.".into(),
            fields: vec![
                field_select(
                    "hourFormat",
                    "Hour format",
                    &[("24h", "24-hour"), ("12h", "12-hour")],
                    "24h",
                ),
                field_bool("showSeconds", "Show seconds", true),
                field_bool("showDate", "Show date", true),
            ],
        },
        // --- glassout integration -------------------------------------------
        // A glassout view shows a live MSFS panel captured by a glassout engine
        // (https://glassout.flyingart.dev). The kiosk window navigates directly
        // to the engine's HTML viewer URL, built from these settings by
        // `glassout::build_view_url`.
        //
        // The admin app uses the `glassout-client` SDK to discover engines on
        // the LAN and enumerate panels, so an operator picks `engineUrl` and
        // `panelId` from real lists rather than typing them.
        ViewDescriptor {
            id: "glassout".into(),
            name: "Glassout panel".into(),
            description: "Show a live MSFS panel from a glassout engine.".into(),
            fields: vec![
                field_text("engineUrl", "Engine URL", "http://127.0.0.1:8787"),
                field_text("panelId", "Panel id", "PFD_Captain"),
                field_select(
                    "fit",
                    "Fit",
                    &[
                        ("contain", "Contain (letterbox)"),
                        ("stretch", "Stretch (fill)"),
                        ("native", "Native (1:1)"),
                    ],
                    "contain",
                ),
                field_text("targetFps", "Target FPS (10–120)", "30"),
                field_bool("debug", "Latency overlay", false),
                // Advanced: a ready-made viewer path (e.g. "/canvas?..." or
                // "/instance/...") produced by the admin via the SDK URL
                // builders. Overrides the panel fields when set.
                field_text("path", "Advanced: viewer path", ""),
            ],
        },
    ]
}
