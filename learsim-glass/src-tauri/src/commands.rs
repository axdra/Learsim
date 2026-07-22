//! Tauri commands invoked from the glass frontend.

use crate::config::ScreenState;
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

/// Return the current state (assigned view + settings) for the given screen.
/// The frontend calls this with its own window label as `screenId`.
#[tauri::command]
pub fn get_screen_state(
    screen_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<ScreenState, String> {
    let cfg = state.config.lock().unwrap();
    let screen = cfg
        .screen(&screen_id)
        .ok_or_else(|| format!("screen '{screen_id}' not found"))?;
    Ok(ScreenState::from_screen(screen, &cfg.device.name))
}
