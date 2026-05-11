//! Smoke tests for IPC commands using tauri's `MockRuntime`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use pimple_core::event::{DeleteEventRequest, RecurringScope};
use pimple_core::{AppConfig, CollectionId, WeekStart, WindowGeometry};
use sha2::{Digest, Sha256};
use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};
use tauri::{App, Manager};
use tempfile::TempDir;

use pimple_tauri::commands;
use pimple_tauri::error::IpcError;
use pimple_tauri::state::AppState;

fn build_app() -> App<MockRuntime> {
    mock_builder()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::set_vdir_root,
            commands::list_collections,
            commands::events_in_range,
            commands::create_event,
            commands::delete_event,
            commands::get_config,
            commands::set_config,
        ])
        .build(mock_context(noop_assets()))
        .unwrap()
}

fn hex_sha256(s: &str) -> String {
    hex::encode(Sha256::digest(s.as_bytes()))
}

const WEEKLY_ICS: &str = include_str!("../../pimple-core/tests/fixtures/ics/weekly.ics");

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

#[tokio::test]
async fn auto_restore_returns_false_when_no_config() {
    let tmp = TempDir::new().unwrap();
    let app = build_app_with_config_dir(tmp.path().to_owned());
    let state = app.state::<AppState>();

    let restored = commands::auto_restore_vdir(&state).await.unwrap();
    assert!(!restored);
    assert!(state.vdir_root.read().await.is_none());
}

#[tokio::test]
async fn auto_restore_returns_false_when_vdir_root_is_none() {
    let tmp = TempDir::new().unwrap();
    let cfg = AppConfig::default();
    pimple_core::config_store::save(&cfg, tmp.path()).unwrap();

    let app = build_app_with_config_dir(tmp.path().to_owned());
    let state = app.state::<AppState>();

    let restored = commands::auto_restore_vdir(&state).await.unwrap();
    assert!(!restored);
    assert!(state.vdir_root.read().await.is_none());
}

#[tokio::test]
async fn delete_event_all_removes_file_and_index_entry() {
    let tmp = TempDir::new().unwrap();
    let cal = tmp.path().join("personal");
    std::fs::create_dir_all(&cal).unwrap();
    let uid = "standup-1"; // matches weekly.ics
    std::fs::write(cal.join(format!("{uid}.ics")), WEEKLY_ICS).unwrap();

    let app = build_app();
    let state = app.state::<AppState>();
    commands::set_vdir_root(tmp.path().to_string_lossy().into_owned(), state.clone())
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    commands::delete_event(
        DeleteEventRequest {
            uid: uid.to_owned(),
            collection_id: CollectionId::new("personal"),
            expected_raw_hash: hex_sha256(WEEKLY_ICS),
            scope: RecurringScope::All,
            occurrence: None,
        },
        state,
    )
    .await
    .unwrap();

    assert!(!cal.join(format!("{uid}.ics")).exists());
}

#[tokio::test]
async fn delete_event_with_hash_drift_returns_conflict() {
    let tmp = TempDir::new().unwrap();
    let cal = tmp.path().join("personal");
    std::fs::create_dir_all(&cal).unwrap();
    let uid = "standup-1";
    std::fs::write(cal.join(format!("{uid}.ics")), WEEKLY_ICS).unwrap();

    let app = build_app();
    let state = app.state::<AppState>();
    commands::set_vdir_root(tmp.path().to_string_lossy().into_owned(), state.clone())
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    let err = commands::delete_event(
        DeleteEventRequest {
            uid: uid.to_owned(),
            collection_id: CollectionId::new("personal"),
            expected_raw_hash: "0".repeat(64),
            scope: RecurringScope::All,
            occurrence: None,
        },
        state,
    )
    .await
    .unwrap_err();

    assert!(
        matches!(err, IpcError::Conflict { .. }),
        "expected Conflict, got {err:?}"
    );
    assert!(cal.join(format!("{uid}.ics")).exists());
}

#[tokio::test]
async fn auto_restore_starts_watcher_for_persisted_vdir() {
    let tmp = TempDir::new().unwrap();
    let vdir = tmp.path().join("vdir");
    std::fs::create_dir_all(vdir.join("personal")).unwrap();
    std::fs::create_dir_all(vdir.join("work")).unwrap();

    let config_dir = tmp.path().join("config");
    let cfg = AppConfig {
        vdir_root: Some(vdir.clone()),
        ..AppConfig::default()
    };
    pimple_core::config_store::save(&cfg, &config_dir).unwrap();

    let app = build_app_with_config_dir(config_dir);
    let state = app.state::<AppState>();

    let restored = commands::auto_restore_vdir(&state).await.unwrap();
    assert!(restored);
    tokio::time::sleep(Duration::from_millis(150)).await;

    assert_eq!(*state.vdir_root.read().await, Some(vdir));
    let collections = commands::list_collections(state).await.unwrap();
    let mut ids: Vec<_> = collections
        .iter()
        .map(|c| c.id.as_str().to_owned())
        .collect();
    ids.sort();
    assert_eq!(ids, vec!["personal".to_owned(), "work".to_owned()]);
}
