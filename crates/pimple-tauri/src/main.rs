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
        .plugin(tauri_plugin_dialog::init())
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
            forwarder::spawn(handle.clone(), &state.index);

            // Auto-restore the watcher from persisted config. Failure is
            // non-fatal — the frontend falls back to the first-run picker.
            let restore_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                let state = restore_handle.state::<AppState>();
                match commands::auto_restore_vdir(&state).await {
                    Ok(true) => tracing::info!("auto-restored vdir from config"),
                    Ok(false) => tracing::debug!("no persisted vdir; awaiting first-run picker"),
                    Err(e) => tracing::warn!("auto-restore failed: {e}"),
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::set_vdir_root,
            commands::list_collections,
            commands::events_in_range,
            commands::create_event,
            commands::delete_event,
            commands::get_config,
            commands::set_config,
            commands::pick_vdir_root,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start pimple");
}
