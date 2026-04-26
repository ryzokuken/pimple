//! Shared application state held by Tauri.

use std::path::PathBuf;
use std::sync::Arc;

use pimple_core::index::EventIndex;
use pimple_core::watcher::FilesystemWatcher;
use tokio::sync::RwLock;

pub struct AppState {
    pub vdir_root: RwLock<Option<PathBuf>>,
    pub index: EventIndex,
    pub watcher: RwLock<Option<Arc<FilesystemWatcher>>>,
}

impl AppState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            vdir_root: RwLock::new(None),
            index: EventIndex::new(),
            watcher: RwLock::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
