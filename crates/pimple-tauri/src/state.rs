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
    /// Test-only override for the config directory. `None` in production;
    /// production resolves via `dirs::config_dir().join("pimple")`.
    pub config_dir_override: Option<PathBuf>,
}

impl AppState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            vdir_root: RwLock::new(None),
            index: EventIndex::new(),
            watcher: RwLock::new(None),
            config_dir_override: None,
        }
    }

    /// Test constructor that pins the config directory to a caller-supplied
    /// path. Production code uses [`Self::new`].
    #[must_use]
    pub fn with_config_dir(config_dir: PathBuf) -> Self {
        Self {
            vdir_root: RwLock::new(None),
            index: EventIndex::new(),
            watcher: RwLock::new(None),
            config_dir_override: Some(config_dir),
        }
    }

    /// Resolve the directory containing `config.toml`. Tests can override via
    /// [`Self::with_config_dir`]; production falls back to
    /// `dirs::config_dir().join("pimple")`.
    ///
    /// # Errors
    ///
    /// Returns `None` if no override is set and the OS reports no config dir
    /// (e.g., `$HOME` unset on Linux). Callers map to an IPC error.
    #[must_use]
    pub fn resolve_config_dir(&self) -> Option<PathBuf> {
        self.config_dir_override
            .clone()
            .or_else(|| dirs::config_dir().map(|p| p.join("pimple")))
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
