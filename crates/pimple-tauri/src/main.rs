#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use pimple_tauri::{commands, diagnostics, forwarder, geometry, state::AppState};
use tauri::{Manager, WindowEvent};
use tokio::sync::mpsc;
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

            // Restore persisted window geometry, then wire a debounced
            // persistence loop for future resize/move events.
            if let Some(window) = app.get_webview_window("main") {
                if let Some(geom) = geometry::load_geometry(&state) {
                    geometry::apply_geometry(&window, &geom);
                }
                spawn_geometry_persistence(&handle, &window);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::set_vdir_root,
            commands::list_collections,
            commands::events_in_range,
            commands::create_event,
            commands::update_event,
            commands::delete_event,
            commands::get_config,
            commands::set_config,
            commands::pick_vdir_root,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start pimple");
}

/// Wire up a debounced "save current window geometry" loop. The
/// `on_window_event` closure runs synchronously on the main thread and only
/// sends a tick into an mpsc channel; an async task drains, waits 500 ms of
/// quiet, then snapshots and persists. This collapses bursty resize events
/// (the user dragging a corner) into one disk write per gesture.
fn spawn_geometry_persistence(handle: &tauri::AppHandle, window: &tauri::WebviewWindow) {
    let (tx, mut rx) = mpsc::unbounded_channel::<()>();
    window.on_window_event(move |event| match event {
        WindowEvent::Resized(_) | WindowEvent::Moved(_) => {
            let _ = tx.send(());
        }
        _ => {}
    });

    let save_handle = handle.clone();
    let window_for_save = window.clone();
    tauri::async_runtime::spawn(async move {
        while rx.recv().await.is_some() {
            // Coalesce: keep waiting as long as new ticks arrive within 500ms.
            loop {
                let quiet = tokio::time::sleep(Duration::from_millis(500));
                tokio::pin!(quiet);
                tokio::select! {
                    () = &mut quiet => break,
                    next = rx.recv() => {
                        if next.is_none() { return; }
                    }
                }
            }
            let Some(geom) = geometry::current_geometry(&window_for_save) else {
                continue;
            };
            let state = save_handle.state::<AppState>();
            if let Err(e) = geometry::save_geometry(&state, geom) {
                tracing::warn!("save window geometry failed: {e}");
            }
        }
    });
}
