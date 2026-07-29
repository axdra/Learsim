//! Embedded control server — the same LAN API as the Tauri client, so the admin
//! app manages this device identically. On any change it persists and bumps the
//! shared change signal so the glassout supervisor reacts.

use crate::state::Shared;
use crate::views::view_manifest;
use crate::config::ScreenConfig;
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
use tower_http::cors::CorsLayer;

pub async fn run(shared: Arc<Shared>, port: u16) {
    let router = Router::new()
        .route("/api/health", get(health))
        .route("/api/device", get(get_device))
        .route("/api/screens", post(add_screen))
        .route("/api/screens/{id}", axum::routing::put(put_screen).delete(delete_screen))
        .layer(CorsLayer::permissive())
        .with_state(shared);

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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceResponse {
    device: crate::config::DeviceInfo,
    port: u16,
    screens: Vec<ScreenConfig>,
    views: Vec<crate::views::ViewDescriptor>,
}

async fn health() -> impl IntoResponse {
    Json(json!({ "app": "learsim-glass-native", "version": env!("CARGO_PKG_VERSION") }))
}

async fn get_device(State(shared): State<Arc<Shared>>) -> impl IntoResponse {
    let cfg = shared.config.lock().unwrap().clone();
    Json(DeviceResponse {
        device: cfg.device,
        port: cfg.port,
        screens: cfg.screens,
        views: view_manifest(),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateScreen {
    view_id: Option<String>,
    settings: Option<Map<String, Value>>,
    name: Option<String>,
}

async fn put_screen(
    Path(id): Path<String>,
    State(shared): State<Arc<Shared>>,
    Json(body): Json<UpdateScreen>,
) -> Result<Json<ScreenConfig>, (StatusCode, String)> {
    let updated = {
        let mut cfg = shared.config.lock().unwrap();
        let idx = cfg
            .screen_index(&id)
            .ok_or((StatusCode::NOT_FOUND, format!("screen '{id}' not found")))?;
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
        cfg.screens[idx].clone()
    };
    shared.persist();
    shared.bump();
    Ok(Json(updated))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewScreen {
    name: Option<String>,
    monitor: Option<usize>,
}

async fn add_screen(
    State(shared): State<Arc<Shared>>,
    Json(body): Json<NewScreen>,
) -> Json<ScreenConfig> {
    let screen = {
        let mut cfg = shared.config.lock().unwrap();
        let n = cfg.screens.len() + 1;
        let screen = ScreenConfig::new(
            format!("screen-{n}"),
            body.name.unwrap_or_else(|| format!("Screen {n}")),
            body.monitor,
        );
        cfg.screens.push(screen.clone());
        screen
    };
    shared.persist();
    shared.bump();
    Json(screen)
}

async fn delete_screen(
    Path(id): Path<String>,
    State(shared): State<Arc<Shared>>,
) -> Result<StatusCode, (StatusCode, String)> {
    {
        let mut cfg = shared.config.lock().unwrap();
        let idx = cfg
            .screen_index(&id)
            .ok_or((StatusCode::NOT_FOUND, format!("screen '{id}' not found")))?;
        cfg.screens.remove(idx);
    }
    shared.persist();
    shared.bump();
    Ok(StatusCode::NO_CONTENT)
}
