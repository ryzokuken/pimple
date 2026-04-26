#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pimple_tauri::{commands, state::AppState};
use tracing_subscriber::EnvFilter;

#[expect(
    clippy::expect_used,
    reason = "Tauri startup failure is a programmer error in config, not recoverable"
)]
#[expect(
    clippy::exit,
    reason = "tauri::generate_context! macro calls process::exit internally on config errors"
)]
fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::set_vdir_root,
            commands::list_collections,
            commands::events_in_range,
            commands::create_event,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start pimple");
}
