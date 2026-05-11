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
    AppConfig, Collection, CollectionId, CoreError, CreateEventRequest, DeleteEventRequest,
    EventInstance, config_store,
};
use tauri::{AppHandle, Runtime, State};
use tauri_plugin_dialog::DialogExt;

use crate::error::{IpcError, IpcResult};
use crate::state::AppState;

/// Install the given vdir path: start the watcher and update state.
///
/// Shared between [`set_vdir_root`] (user-driven) and
/// [`auto_restore_vdir`] (startup-driven). Performs the same validation in
/// both cases.
///
/// # Errors
///
/// Returns `CoreError::VdirLayout` if `root` is not a directory, or a watcher
/// failure if the FS watch cannot start.
pub async fn install_vdir_root(root: PathBuf, state: &AppState) -> Result<(), CoreError> {
    if !root.is_dir() {
        return Err(CoreError::VdirLayout(format!(
            "not a directory: {}",
            root.display()
        )));
    }
    let watcher = FilesystemWatcher::start(root.clone(), state.index.clone()).await?;
    *state.vdir_root.write().await = Some(root);
    *state.watcher.write().await = Some(Arc::new(watcher));
    Ok(())
}

/// On startup, read the persisted `AppConfig` and reinstall the watcher if
/// `vdir_root` is set. Returns `Ok(true)` if a vdir was restored.
///
/// Silently returns `Ok(false)` when there is no config file or `vdir_root`
/// is `None`; the first-run flow then prompts the user.
///
/// # Errors
///
/// Returns a core error if the config file exists but cannot be parsed, or
/// if installing the watcher fails (typically a now-invalid vdir path).
pub async fn auto_restore_vdir(state: &AppState) -> Result<bool, CoreError> {
    let Some(config_dir) = state.resolve_config_dir() else {
        return Ok(false);
    };
    let cfg = config_store::load(&config_dir)?;
    let Some(root) = cfg.vdir_root else {
        return Ok(false);
    };
    install_vdir_root(root, state).await?;
    Ok(true)
}

/// Set the root vdir directory and start the filesystem watcher.
///
/// # Errors
///
/// Returns [`IpcError::Vdir`] if `path` is not a directory, or a core error
/// if the filesystem watcher fails to start.
#[tauri::command]
pub async fn set_vdir_root(path: String, state: State<'_, AppState>) -> IpcResult<()> {
    install_vdir_root(PathBuf::from(&path), &state)
        .await
        .map_err(IpcError::from)
}

/// Open a native folder picker and return the chosen path, or `None` if the
/// user cancelled. Used by the first-run flow.
///
/// # Errors
///
/// Currently infallible at the IPC layer — the dialog plugin's
/// `blocking_pick_folder` returns `None` on user-cancel rather than erroring.
/// The result type stays `IpcResult` so future fallible variants don't break
/// callers.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "tauri::AppHandle<R> is a thin shared handle; Tauri's command macro requires by-value"
)]
pub fn pick_vdir_root<R: Runtime>(app: AppHandle<R>) -> IpcResult<Option<PathBuf>> {
    let picked = app.dialog().file().blocking_pick_folder();
    Ok(picked.and_then(|fp| fp.into_path().ok()))
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

/// Delete an event, honouring the request's `RecurringScope`.
///
/// # Errors
///
/// Returns [`IpcError::Conflict`] if the on-disk file's hash has drifted from
/// `expected_raw_hash`, [`IpcError::InvalidScope`] for inconsistent scope
/// arguments, or a core error for I/O or parse failures.
#[tauri::command]
pub async fn delete_event(
    request: DeleteEventRequest,
    state: State<'_, AppState>,
) -> IpcResult<()> {
    let root = state.vdir_root.read().await.clone();
    let Some(root) = root else {
        return Err(IpcError::Vdir {
            message: "no vdir configured".into(),
        });
    };
    let collection_path = root.join(request.collection_id.as_str());
    pimple_core::write::delete_event(&collection_path, &request)
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
