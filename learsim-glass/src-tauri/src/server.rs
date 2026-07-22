//! Embedded control server.
//!
//! Each glass device runs this HTTP server (bound on `0.0.0.0:<port>`) so the
//! admin app — typically on another machine — can enumerate the device's
//! screens and reassign views over the LAN. Changes are persisted, the affected
//! kiosk window is navigated to the right content, and a `screen-changed` event
//! is emitted so local views update live without a restart.
//!
//! Endpoints:
//!   GET    /api/health           liveness + version
//!   GET    /api/device           device info, screens, and view manifest
//!   PUT    /api/screens/{id}     reassign / rename a screen (partial update)
//!   POST   /api/screens          add a screen (+ its kiosk window)
//!   DELETE /api/screens/{id}     remove a screen (+ close its window)

use crate::config::{save, ScreenConfig, ScreenState};
use crate::state::AppState;
use crate::views::view_manifest;
use crate::windows;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tower_http::cors::CorsLayer;

/// Everything a request handler needs: shared state + a Tauri handle for
/// emitting live updates and creating/closing/navigating windows.
pub struct ServerCtx {
    pub state: Arc<AppState>,
    pub app: AppHandle,
}

pub async fn run(ctx: Arc<ServerCtx>, port: u16) {
    let router = Router::new()
        .route("/api/health", get(health))
        .route("/api/device", get(get_device))
        .route("/api/screens", post(add_screen))
        .route(
            "/api/screens/{id}",
            axum::routing::put(put_screen).delete(delete_screen),
        )
        .layer(CorsLayer::permissive())
        .with_state(ctx);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            println!("[server] control server listening on http://{addr}");
            if let Err(err) = axum::serve(listener, router).await {
                eprintln!("[server] serve error: {err}");
            }
        }
        Err(err) => eprintln!("[server] could not bind {addr}: {err}"),
    }
}

// --- responses --------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceResponse {
    device: crate::config::DeviceInfo,
    port: u16,
    screens: Vec<ScreenConfig>,
    views: Vec<crate::views::ViewDescriptor>,
}

async fn health() -> impl IntoResponse {
    Json(json!({
        "app": "learsim-glass",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn get_device(State(ctx): State<Arc<ServerCtx>>) -> impl IntoResponse {
    let cfg = ctx.state.config.lock().unwrap().clone();
    Json(DeviceResponse {
        device: cfg.device,
        port: cfg.port,
        screens: cfg.screens,
        views: view_manifest(),
    })
}

// --- screen mutations -------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateScreen {
    view_id: Option<String>,
    settings: Option<Map<String, Value>>,
    name: Option<String>,
}

async fn put_screen(
    Path(id): Path<String>,
    State(ctx): State<Arc<ServerCtx>>,
    Json(body): Json<UpdateScreen>,
) -> Result<Json<ScreenConfig>, (StatusCode, String)> {
    let (updated, device_name) = {
        let mut cfg = ctx.state.config.lock().unwrap();
        let idx = cfg
            .screen_index(&id)
            .ok_or((StatusCode::NOT_FOUND, format!("screen '{id}' not found")))?;
        {
            let screen = &mut cfg.screens[idx];
            if let Some(v) = body.view_id {
                screen.view_id = v;
            }
            if let Some(s) = body.settings {
                screen.settings = s;
            }
            if let Some(n) = body.name {
                screen.name = n;
            }
        }
        let updated = cfg.screens[idx].clone();
        let device_name = cfg.device.name.clone();
        if let Err(err) = save(&ctx.state.config_path, &cfg) {
            eprintln!("[server] persist failed: {err}");
        }
        (updated, device_name)
    };

    // Live-update local views in place…
    let screen_state = ScreenState::from_screen(&updated, &device_name);
    let _ = ctx.app.emit("screen-changed", &screen_state);
    // …and navigate the window when crossing into/out of a glassout view.
    dispatch_apply(&ctx, updated.clone());

    Ok(Json(updated))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewScreen {
    name: Option<String>,
    monitor: Option<usize>,
}

async fn add_screen(
    State(ctx): State<Arc<ServerCtx>>,
    Json(body): Json<NewScreen>,
) -> Json<ScreenConfig> {
    let screen = {
        let mut cfg = ctx.state.config.lock().unwrap();
        let n = cfg.screens.len() + 1;
        let id = format!("screen-{n}");
        let name = body.name.unwrap_or_else(|| format!("Screen {n}"));
        let screen = ScreenConfig::new(id, name, body.monitor);
        cfg.screens.push(screen.clone());
        if let Err(err) = save(&ctx.state.config_path, &cfg) {
            eprintln!("[server] persist failed: {err}");
        }
        screen
    };

    // Create the kiosk window on the main thread.
    let app = ctx.app.clone();
    let screen_for_window = screen.clone();
    let _ = ctx.app.run_on_main_thread(move || {
        if let Err(err) = windows::create_screen_window(&app, &screen_for_window) {
            eprintln!("[server] could not create window: {err}");
        }
    });

    Json(screen)
}

async fn delete_screen(
    Path(id): Path<String>,
    State(ctx): State<Arc<ServerCtx>>,
) -> Result<StatusCode, (StatusCode, String)> {
    {
        let mut cfg = ctx.state.config.lock().unwrap();
        let idx = cfg
            .screen_index(&id)
            .ok_or((StatusCode::NOT_FOUND, format!("screen '{id}' not found")))?;
        cfg.screens.remove(idx);
        if let Err(err) = save(&ctx.state.config_path, &cfg) {
            eprintln!("[server] persist failed: {err}");
        }
    }

    let app = ctx.app.clone();
    let screen_id = id.clone();
    let _ = ctx
        .app
        .run_on_main_thread(move || windows::close_screen_window(&app, &screen_id));

    Ok(StatusCode::NO_CONTENT)
}

/// Navigate a screen's window to match its (possibly changed) view, on the
/// main thread.
fn dispatch_apply(ctx: &Arc<ServerCtx>, screen: ScreenConfig) {
    let app = ctx.app.clone();
    let app_url = ctx.state.app_url.lock().unwrap().clone();
    let _ = ctx.app.run_on_main_thread(move || {
        windows::apply_screen_view(&app, &screen, app_url);
    });
}
