//! Tauri IPC commands.
//!
//! Each command is a thin wrapper that converts `pimple_core::Result` into
//! `IpcResult` for the frontend.

use std::path::PathBuf;
use std::sync::Arc;

use jiff::Timestamp;
use pimple_core::vdir::layout::enumerate_collections;
use pimple_core::watcher::FilesystemWatcher;
use pimple_core::{
    AppConfig, Collection, CollectionId, CreateEventRequest, EventInstance, config_store,
};
use tauri::State;

use crate::error::{IpcError, IpcResult};
use crate::state::AppState;

/// Set the root vdir directory and start the filesystem watcher.
///
/// # Errors
///
/// Returns [`IpcError::Vdir`] if `path` is not a directory, or a core error
/// if the filesystem watcher fails to start.
#[tauri::command]
pub async fn set_vdir_root(path: String, state: State<'_, AppState>) -> IpcResult<()> {
    let root = PathBuf::from(&path);
    if !root.is_dir() {
        return Err(IpcError::Vdir {
            message: format!("not a directory: {path}"),
        });
    }
    {
        let mut current = state.vdir_root.write().await;
        *current = Some(root.clone());
    }
    let watcher = FilesystemWatcher::start(root, state.index.clone())
        .await
        .map_err(IpcError::from)?;
    let mut slot = state.watcher.write().await;
    *slot = Some(Arc::new(watcher));
    Ok(())
}

/// List all collections under the configured vdir root.
///
/// Returns an empty list if no vdir root is set.
///
/// # Errors
///
/// Returns a core error if the collection directory cannot be read.
#[tauri::command]
pub async fn list_collections(state: State<'_, AppState>) -> IpcResult<Vec<Collection>> {
    let root = state.vdir_root.read().await.clone();
    let Some(root) = root else {
        return Ok(Vec::new());
    };
    enumerate_collections(&root).map_err(IpcError::from)
}

/// Return events whose occurrences overlap with `[start, end)`.
///
/// # Errors
///
/// Returns [`IpcError::Ical`] if `start` or `end` cannot be parsed as
/// RFC 3339 timestamps, or a core error from the index.
#[tauri::command]
pub async fn events_in_range(
    start: String,
    end: String,
    visible_collections: Vec<String>,
    state: State<'_, AppState>,
) -> IpcResult<Vec<EventInstance>> {
    let start: Timestamp = start.parse().map_err(|e: jiff::Error| IpcError::Ical {
        message: format!("start parse: {e}"),
    })?;
    let end: Timestamp = end.parse().map_err(|e: jiff::Error| IpcError::Ical {
        message: format!("end parse: {e}"),
    })?;
    let visible: Vec<CollectionId> = visible_collections
        .into_iter()
        .map(CollectionId::new)
        .collect();
    state
        .index
        .events_in_range(start, end, &visible)
        .await
        .map_err(IpcError::from)
}

/// Create a new event in the specified collection.
///
/// Returns the UID of the newly created event.
///
/// # Errors
///
/// Returns [`IpcError::Vdir`] if no vdir root is configured or the collection
/// directory does not exist, or a core error if writing the `.ics` file fails.
#[tauri::command]
pub async fn create_event(
    request: CreateEventRequest,
    state: State<'_, AppState>,
) -> IpcResult<String> {
    let root = state.vdir_root.read().await.clone();
    let Some(root) = root else {
        return Err(IpcError::Vdir {
            message: "no vdir configured".into(),
        });
    };
    let collection_path = root.join(request.collection_id.as_str());
    pimple_core::write::create_event(&collection_path, &request)
        .await
        .map_err(IpcError::from)
}

/// Read the persisted `AppConfig` from disk. Returns defaults if the file is
/// absent — `get_config` never errors on a missing file because the first
/// launch legitimately has no config yet.
///
/// # Errors
///
/// Returns [`IpcError::Internal`] if the OS cannot identify a config
/// directory, or a core error if the file exists but cannot be parsed.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "tauri::State<'_, T> is a thin shared handle; Tauri's command macro requires by-value"
)]
pub fn get_config(state: State<'_, AppState>) -> IpcResult<AppConfig> {
    let dir = state
        .resolve_config_dir()
        .ok_or_else(|| IpcError::Internal {
            message: "could not resolve OS config directory".into(),
        })?;
    config_store::load(&dir).map_err(IpcError::from)
}

/// Persist the given `AppConfig` to disk atomically.
///
/// # Errors
///
/// Returns [`IpcError::Internal`] if the OS cannot identify a config
/// directory, or a core error if the write fails.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "tauri::State<'_, T> is a thin shared handle; Tauri's command macro requires by-value"
)]
pub fn set_config(config: AppConfig, state: State<'_, AppState>) -> IpcResult<()> {
    let dir = state
        .resolve_config_dir()
        .ok_or_else(|| IpcError::Internal {
            message: "could not resolve OS config directory".into(),
        })?;
    config_store::save(&config, &dir).map_err(IpcError::from)
}
