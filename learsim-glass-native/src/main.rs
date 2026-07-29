//! learsim-glass-native — a native (no-webview) glass display client.
//!
//! Renders glassout panels and local views directly with SDL2 (KMSDRM on a
//! headless Pi), speaking the glassout WebSocket protocol itself. It runs the
//! same control server as the Tauri client, so the admin app is unchanged.
//!
//! Threading: SDL2 owns the main thread (render loop); a background tokio
//! runtime hosts the control server and the glassout supervisor. They share the
//! config (Mutex), the latest decoded frame (Mutex), a config-change watch, and
//! an input channel.

mod config;
mod glassout;
mod render;
mod server;
mod state;
mod types;
mod views;

use state::Shared;
use std::sync::{Arc, Mutex};
use types::LatestFrame;

fn main() {
    // On a bare console (no X/Wayland) use SDL's KMSDRM backend so we draw
    // straight to the display with no compositor.
    if std::env::var_os("DISPLAY").is_none()
        && std::env::var_os("WAYLAND_DISPLAY").is_none()
        && std::env::var_os("SDL_VIDEODRIVER").is_none()
    {
        std::env::set_var("SDL_VIDEODRIVER", "kmsdrm");
    }

    let config_path = config::default_path();
    let cfg = config::load(&config_path);
    let port = cfg.port;
    // The native client renders one screen — the first configured one.
    let screen_id = cfg
        .screens
        .first()
        .map(|s| s.id.clone())
        .unwrap_or_else(|| "screen-1".to_string());

    let shared = Arc::new(Shared::new(cfg, config_path));
    shared.persist(); // write out a freshly-created default, if any

    let latest: LatestFrame = Arc::new(Mutex::new(None));
    let (input_tx, input_rx) = tokio::sync::mpsc::unbounded_channel();

    // Background async runtime: control server + glassout supervisor.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    rt.spawn({
        let shared = shared.clone();
        async move { server::run(shared, port).await }
    });
    rt.spawn({
        let shared = shared.clone();
        let latest = latest.clone();
        let screen_id = screen_id.clone();
        async move { glassout::supervise(shared, screen_id, latest, input_rx).await }
    });

    // Render loop owns the main thread until the window closes.
    if let Err(err) = render::run(shared, screen_id, latest, input_tx) {
        eprintln!("[render] fatal: {err}");
        std::process::exit(1);
    }
}
