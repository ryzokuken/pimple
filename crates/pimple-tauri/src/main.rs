#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pimple_tauri::{commands, diagnostics, forwarder, state::AppState};
use tauri::Manager;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[expect(
    clippy::expect_used,
    reason = "Tauri startup failure is a programmer error in config, not recoverable"
)]
#[expect(
    clippy::exit,
    reason = "tauri::generate_context! macro calls process::exit internally on config errors"
)]
fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .setup(|app| {
            let handle = app.handle().clone();

            tracing_subscriber::registry()
                .with(EnvFilter::from_default_env())
                .with(tracing_subscriber::fmt::layer())
                .with(diagnostics::ForwardingLayer {
                    handle: handle.clone(),
                })
                .init();

            let state = app.state::<AppState>();
            forwarder::spawn(handle, &state.index);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::set_vdir_root,
            commands::list_collections,
            commands::events_in_range,
            commands::create_event,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start pimple");
}
