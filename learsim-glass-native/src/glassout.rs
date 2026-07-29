//! Native glassout client: speaks the engine's WebSocket protocol directly,
//! decodes JPEG panel frames, and forwards taps as `panel.input`.
//!
//! Wire format of a binary frame (little-endian), per the engine protocol:
//!   [u8 panelId len N][N bytes panelId][u32 frameNumber][u16 w][u16 h]
//!   [u64 engineTimestampMs][JPEG bytes...]

use crate::state::Shared;
use crate::types::{DecodedFrame, InputEvent, LatestFrame};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tokio_tungstenite::tungstenite::Message;
use url::Url;
use zune_jpeg::zune_core::colorspace::ColorSpace;
use zune_jpeg::zune_core::options::DecoderOptions;
use zune_jpeg::JpegDecoder;

/// Parse a binary frame into (panel id, width, height, jpeg bytes).
fn parse_frame(buf: &[u8]) -> Option<(u16, u16, &[u8])> {
    let n = *buf.first()? as usize;
    let mut off = 1usize;
    // panelId (n) + frameNumber(4) + w(2) + h(2) + ts(8) = n + 16
    if buf.len() < off + n + 16 {
        return None;
    }
    off += n; // skip panelId (we already know which panel we subscribed to)
    off += 4; // frameNumber
    let width = u16::from_le_bytes([buf[off], buf[off + 1]]);
    off += 2;
    let height = u16::from_le_bytes([buf[off], buf[off + 1]]);
    off += 2;
    off += 8; // engineTimestampMs
    Some((width, height, &buf[off..]))
}

fn decode_jpeg(jpeg: &[u8]) -> Option<DecodedFrame> {
    let opts = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGB);
    let mut dec = JpegDecoder::new_with_options(jpeg, opts);
    let rgb = dec.decode().ok()?;
    let (w, h) = dec.dimensions()?;
    Some(DecodedFrame {
        width: w as u32,
        height: h as u32,
        rgb,
    })
}

/// What the supervisor should render/connect to for the target screen.
enum Target {
    Glassout {
        engine_url: String,
        panel_id: String,
        target_fps: u64,
        click_delay: Option<u64>,
    },
    Local,
}

fn read_target(shared: &Shared, screen_id: &str) -> Target {
    let cfg = shared.config.lock().unwrap();
    let Some(screen) = cfg.screen(screen_id) else {
        return Target::Local;
    };
    if screen.view_id != "glassout" {
        return Target::Local;
    }
    match (screen.setting_str("engineUrl"), screen.setting_str("panelId")) {
        (Some(engine), Some(panel)) => Target::Glassout {
            engine_url: engine.to_string(),
            panel_id: panel.to_string(),
            target_fps: screen.setting_u64("targetFps").unwrap_or(30).clamp(10, 120),
            click_delay: screen.setting_u64("clickDelay"),
        },
        _ => Target::Local,
    }
}

enum Outcome {
    ConfigChanged,
    Disconnected,
}

fn build_ws_url(engine_url: &str) -> Option<String> {
    let u = Url::parse(engine_url).ok()?;
    let host = u.host_str()?;
    let port = u.port().unwrap_or(8787);
    Some(format!(
        "ws://{host}:{port}/ws?name=learsim-glass&appKey=learsim-glass&connectionType=viewer"
    ))
}

async fn run_session(
    engine_url: &str,
    panel_id: &str,
    target_fps: u64,
    click_delay: Option<u64>,
    latest: &LatestFrame,
    input_rx: &mut mpsc::UnboundedReceiver<InputEvent>,
    change_rx: &mut watch::Receiver<u64>,
) -> Outcome {
    let Some(ws_url) = build_ws_url(engine_url) else {
        eprintln!("[glassout] bad engine URL: {engine_url}");
        return Outcome::Disconnected;
    };

    let (ws, _) = match tokio_tungstenite::connect_async(ws_url.as_str()).await {
        Ok(pair) => pair,
        Err(err) => {
            eprintln!("[glassout] connect {ws_url} failed: {err}");
            return Outcome::Disconnected;
        }
    };
    println!("[glassout] connected to {ws_url}, panel '{panel_id}'");
    let (mut write, mut read) = ws.split();

    let subscribe = serde_json::json!({
        "type": "subscribe.panel",
        "panelId": panel_id,
        "targetFps": target_fps,
    })
    .to_string();
    let mut subscribed = false;

    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        if let Some((_, _, jpeg)) = parse_frame(&data) {
                            if let Some(frame) = decode_jpeg(jpeg) {
                                *latest.lock().unwrap() = Some(frame);
                            }
                        }
                    }
                    Some(Ok(Message::Text(text))) => {
                        // Subscribe once we've seen the welcome frame.
                        if !subscribed {
                            let is_welcome = serde_json::from_str::<serde_json::Value>(text.as_str())
                                .ok()
                                .and_then(|v| v.get("type").and_then(|t| t.as_str()).map(|s| s == "welcome"))
                                .unwrap_or(false);
                            if is_welcome {
                                if write.send(Message::Text(subscribe.clone().into())).await.is_err() {
                                    return Outcome::Disconnected;
                                }
                                subscribed = true;
                            }
                        }
                    }
                    Some(Ok(Message::Ping(p))) => {
                        let _ = write.send(Message::Pong(p)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => return Outcome::Disconnected,
                    Some(Ok(_)) => {}
                    Some(Err(err)) => {
                        eprintln!("[glassout] read error: {err}");
                        return Outcome::Disconnected;
                    }
                }
            }
            Some(ev) = input_rx.recv() => {
                if subscribed {
                    let mut msg = serde_json::json!({
                        "type": "panel.input",
                        "panelId": panel_id,
                        "x": ev.x,
                        "y": ev.y,
                        "kind": "click",
                    });
                    if let Some(delay) = click_delay {
                        msg["delayMs"] = serde_json::json!(delay);
                    }
                    let _ = write.send(Message::Text(msg.to_string().into())).await;
                }
            }
            _ = change_rx.changed() => {
                return Outcome::ConfigChanged;
            }
        }
    }
}

/// Long-running task: keeps the target screen's glassout panel connected and
/// decoded into `latest`, reconnecting on drops and reacting to admin changes.
pub async fn supervise(
    shared: Arc<Shared>,
    screen_id: String,
    latest: LatestFrame,
    mut input_rx: mpsc::UnboundedReceiver<InputEvent>,
) {
    let mut change_rx = shared.change_tx.subscribe();

    loop {
        match read_target(&shared, &screen_id) {
            Target::Glassout {
                engine_url,
                panel_id,
                target_fps,
                click_delay,
            } => {
                let outcome = run_session(
                    &engine_url,
                    &panel_id,
                    target_fps,
                    click_delay,
                    &latest,
                    &mut input_rx,
                    &mut change_rx,
                )
                .await;

                match outcome {
                    Outcome::ConfigChanged => continue,
                    Outcome::Disconnected => {
                        // Show the placeholder and retry after a short backoff,
                        // but wake immediately if the config changes.
                        *latest.lock().unwrap() = None;
                        tokio::select! {
                            _ = tokio::time::sleep(Duration::from_secs(2)) => {}
                            _ = change_rx.changed() => {}
                        }
                    }
                }
            }
            Target::Local => {
                *latest.lock().unwrap() = None;
                // Nothing to stream; wait for the next config change.
                if change_rx.changed().await.is_err() {
                    return;
                }
            }
        }
    }
}
