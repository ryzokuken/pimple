//! Forward `pimple_core` index changes as Tauri events.

use pimple_core::index::{EventIndex, IndexChange};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};
use tokio::sync::broadcast::error::RecvError;
use tracing::{debug, warn};

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventsChangedPayload {
    Upsert { uid: String },
    Remove { uid: String },
    FullReload,
}

impl From<IndexChange> for EventsChangedPayload {
    fn from(value: IndexChange) -> Self {
        match value {
            IndexChange::Upsert { uid, .. } => Self::Upsert { uid },
            IndexChange::Remove { uid } => Self::Remove { uid },
            IndexChange::FullReload { .. } => Self::FullReload,
        }
    }
}

/// Spawn the forwarder. Lives for the lifetime of the app.
pub fn spawn<R: Runtime>(handle: AppHandle<R>, index: &EventIndex) {
    let mut rx = index.subscribe();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(change) => {
                    let payload: EventsChangedPayload = change.into();
                    if let Err(e) = handle.emit("events_changed", payload) {
                        warn!("emit events_changed failed: {e}");
                    }
                }
                Err(RecvError::Lagged(n)) => {
                    debug!("forwarder lagged by {n} messages; emitting full reload");
                    if let Err(e) = handle.emit("events_changed", EventsChangedPayload::FullReload) {
                        warn!("emit lag-recovery FullReload failed: {e}");
                    }
                }
                Err(RecvError::Closed) => break,
            }
        }
    });
}
