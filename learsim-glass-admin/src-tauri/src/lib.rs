//! learsim-glass-admin — configure which view each glass screen shows.
//!
//! The UI talks to glass devices over their LAN control servers (proxied
//! through the commands in `client.rs`) and, for glassout views, uses the
//! `glassout-client` SDK on the frontend to discover engines and panels.

mod client;
mod devices;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state = client::build_state(app);
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            client::list_devices,
            client::add_device,
            client::remove_device,
            client::ping_device,
            client::fetch_device,
            client::set_screen,
            client::add_screen,
            client::delete_screen,
            client::probe_engine,
        ])
        .run(tauri::generate_context!())
        .expect("error while running learsim-glass-admin");
}
