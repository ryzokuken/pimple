//! Window geometry persistence helpers.
//!
//! `current_geometry` snapshots a window's position and size; `apply_geometry`
//! restores them at startup. `save_geometry` mutates the persisted
//! `AppConfig.window_geometry` field through the config store.
//!
//! The debouncer that ties window events to disk writes lives in `main.rs`,
//! because it needs the live `AppHandle` and the Tokio runtime.

use pimple_core::{WindowGeometry, config_store};
use tauri::{PhysicalPosition, PhysicalSize, Runtime, WebviewWindow};

use crate::error::{IpcError, IpcResult};
use crate::state::AppState;

/// Snapshot the window's outer position and inner size as a `WindowGeometry`.
/// Returns `None` if Tauri can't read either (uncommon — usually a closing
/// window that's already been torn down).
#[must_use]
pub fn current_geometry<R: Runtime>(window: &WebviewWindow<R>) -> Option<WindowGeometry> {
    let size = window.inner_size().ok()?;
    let pos = window.outer_position().ok()?;
    Some(WindowGeometry {
        width: size.width,
        height: size.height,
        x: pos.x,
        y: pos.y,
    })
}

/// Restore a window to the geometry recorded by `current_geometry`. Failures
/// are silently swallowed (best-effort restore — losing window geometry isn't
/// worth crashing the app).
pub fn apply_geometry<R: Runtime>(window: &WebviewWindow<R>, geom: &WindowGeometry) {
    let _ = window.set_size(PhysicalSize::new(geom.width, geom.height));
    let _ = window.set_position(PhysicalPosition::new(geom.x, geom.y));
}

/// Persist a `WindowGeometry` into the user's `AppConfig`, leaving every
/// other field intact.
///
/// # Errors
///
/// Returns [`IpcError::Internal`] if the OS provides no config directory, or
/// a core error if reading/writing the config file fails.
pub fn save_geometry(state: &AppState, geom: WindowGeometry) -> IpcResult<()> {
    let dir = state
        .resolve_config_dir()
        .ok_or_else(|| IpcError::Internal {
            message: "could not resolve OS config directory".into(),
        })?;
    let mut config = config_store::load(&dir).map_err(IpcError::from)?;
    config.window_geometry = Some(geom);
    config_store::save(&config, &dir).map_err(IpcError::from)
}

/// Read the persisted `WindowGeometry` from the user's `AppConfig`, or `None`
/// if either no config exists or it has no geometry recorded.
#[must_use]
pub fn load_geometry(state: &AppState) -> Option<WindowGeometry> {
    let dir = state.resolve_config_dir()?;
    config_store::load(&dir).ok()?.window_geometry
}
