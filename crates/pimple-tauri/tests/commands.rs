//! Smoke tests for IPC commands using tauri's `MockRuntime`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::Duration;

use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};
use tauri::{App, Manager};
use tempfile::TempDir;

use pimple_tauri::commands;
use pimple_tauri::state::AppState;

fn build_app() -> App<MockRuntime> {
    mock_builder()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::set_vdir_root,
            commands::list_collections,
            commands::events_in_range,
            commands::create_event,
        ])
        .build(mock_context(noop_assets()))
        .unwrap()
}

#[tokio::test]
async fn set_vdir_root_then_list_collections_returns_subdirs() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join("personal")).unwrap();
    std::fs::create_dir_all(tmp.path().join("work")).unwrap();

    let app = build_app();
    let state: tauri::State<'_, AppState> = app.state::<AppState>();

    commands::set_vdir_root(tmp.path().to_string_lossy().into_owned(), state.clone())
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(150)).await;

    let collections = commands::list_collections(state).await.unwrap();
    let mut ids: Vec<_> = collections
        .iter()
        .map(|c| c.id.as_str().to_owned())
        .collect();
    ids.sort();
    assert_eq!(ids, vec!["personal".to_owned(), "work".to_owned()]);
}
