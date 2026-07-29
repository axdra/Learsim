//! View manifest — identical to the Tauri client so the admin's UI is the same.

use serde::Serialize;
use serde_json::{json, Value};

pub const DEFAULT_VIEW_ID: &str = "standby";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewField {
    pub key: String,
    pub label: String,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewDescriptor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub fields: Vec<ViewField>,
}

fn text(key: &str, label: &str, default: &str) -> ViewField {
    ViewField { key: key.into(), label: label.into(), kind: "text".into(), options: vec![], default: json!(default) }
}
fn boolean(key: &str, label: &str, default: bool) -> ViewField {
    ViewField { key: key.into(), label: label.into(), kind: "boolean".into(), options: vec![], default: json!(default) }
}
fn select(key: &str, label: &str, opts: &[(&str, &str)], default: &str) -> ViewField {
    ViewField {
        key: key.into(),
        label: label.into(),
        kind: "select".into(),
        options: opts.iter().map(|(v, l)| ViewOption { value: (*v).into(), label: (*l).into() }).collect(),
        default: json!(default),
    }
}

pub fn view_manifest() -> Vec<ViewDescriptor> {
    vec![
        ViewDescriptor {
            id: "standby".into(),
            name: "Standby".into(),
            description: "Idle screen showing the device and screen name.".into(),
            fields: vec![text("message", "Message", "")],
        },
        ViewDescriptor {
            id: "clock".into(),
            name: "Clock".into(),
            description: "Large clock with optional seconds, date, and UTC/Zulu.".into(),
            fields: vec![
                select("hourFormat", "Hour format", &[("24h", "24-hour"), ("12h", "12-hour")], "24h"),
                boolean("showSeconds", "Show seconds", true),
                boolean("showDate", "Show date", true),
                boolean("utc", "UTC / Zulu time", false),
            ],
        },
        ViewDescriptor {
            id: "testpattern".into(),
            name: "Test pattern".into(),
            description: "Alignment grid, edge frame, and colour bars for setting up panels.".into(),
            fields: vec![],
        },
        ViewDescriptor {
            id: "glassout".into(),
            name: "Glassout panel".into(),
            description: "Show a live MSFS panel from a glassout engine (rendered natively).".into(),
            fields: vec![
                text("engineUrl", "Engine URL", "http://127.0.0.1:8787"),
                text("panelId", "Panel id", "PFD_Captain"),
                select(
                    "fit",
                    "Fit",
                    &[("contain", "Contain (letterbox)"), ("stretch", "Stretch (fill)"), ("native", "Native (1:1)")],
                    "contain",
                ),
                text("targetFps", "Target FPS (10–120)", "30"),
                text("clickDelay", "Click delay ms (blank = engine default)", ""),
            ],
        },
    ]
}
