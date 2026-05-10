//! Smoke tests for IPC commands using tauri's `MockRuntime`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use pimple_core::{AppConfig, WeekStart, WindowGeometry};
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
            commands::get_config,
            commands::set_config,
        ])
        .build(mock_context(noop_assets()))
        .unwrap()
}

fn build_app_with_config_dir(config_dir: PathBuf) -> App<MockRuntime> {
    mock_builder()
        .manage(AppState::with_config_dir(config_dir))
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_config
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

#[test]
fn get_config_returns_defaults_on_first_launch() {
    let tmp = TempDir::new().unwrap();
    let app = build_app_with_config_dir(tmp.path().to_owned());
    let state = app.state::<AppState>();

    let cfg = commands::get_config(state).unwrap();
    assert!(cfg.vdir_root.is_none());
    assert!(cfg.collection_visibility.is_empty());
    assert_eq!(cfg.week_start, WeekStart::Monday);
    assert!(cfg.window_geometry.is_none());
}

#[test]
fn set_then_get_config_round_trips() {
    let tmp = TempDir::new().unwrap();
    let app = build_app_with_config_dir(tmp.path().to_owned());
    let state = app.state::<AppState>();

    let mut visibility = HashMap::new();
    visibility.insert("personal".to_owned(), true);
    let original = AppConfig {
        vdir_root: Some(PathBuf::from("/home/me/.calendars")),
        collection_visibility: visibility,
        week_start: WeekStart::Sunday,
        window_geometry: Some(WindowGeometry {
            width: 1100,
            height: 720,
            x: 0,
            y: 0,
        }),
    };

    commands::set_config(original.clone(), state.clone()).unwrap();
    let loaded = commands::get_config(state).unwrap();
    assert_eq!(loaded, original);
}
