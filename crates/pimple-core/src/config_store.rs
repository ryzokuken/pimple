//! Read/write `AppConfig` to a TOML file under an OS-appropriate directory.
//!
//! `pimple-core` is platform-agnostic by design: callers (the Tauri layer)
//! resolve the actual config directory via the `dirs` crate and pass it in. A
//! missing file yields `AppConfig::default()`; unknown TOML fields are
//! silently ignored so forward-compatible configs from a future version still
//! load cleanly. See v0.2 spec §5.1.

use std::fs;
use std::path::Path;

use crate::config::AppConfig;
use crate::error::{CoreError, Result};

const CONFIG_FILENAME: &str = "config.toml";

/// Load `AppConfig` from `<config_dir>/config.toml`. Returns the default
/// config if the file is absent.
///
/// # Errors
///
/// Returns `CoreError::Io` if the file exists but cannot be read, and
/// `CoreError::Conversion` (mapped from a TOML parse failure) if the
/// contents are not valid TOML.
pub fn load(config_dir: &Path) -> Result<AppConfig> {
    let path = config_dir.join(CONFIG_FILENAME);
    match fs::read_to_string(&path) {
        Ok(text) => toml::from_str(&text)
            .map_err(|e| CoreError::Conversion(format!("config.toml parse: {e}"))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(AppConfig::default()),
        Err(e) => Err(e.into()),
    }
}

/// Atomically save `AppConfig` to `<config_dir>/config.toml`.
///
/// Creates `config_dir` (and any missing parents) if absent. Writes to a
/// sibling temp file and renames into place, matching the durability pattern
/// used by `pimple-core::write::create_event`.
///
/// # Errors
///
/// Returns `CoreError::Io` if the directory cannot be created, the temp file
/// cannot be written, or the rename fails. `CoreError::Conversion` if TOML
/// serialization itself fails (should not happen in practice).
pub fn save(config: &AppConfig, config_dir: &Path) -> Result<()> {
    fs::create_dir_all(config_dir)?;
    let text = toml::to_string_pretty(config)
        .map_err(|e| CoreError::Conversion(format!("config.toml serialize: {e}")))?;
    let final_path = config_dir.join(CONFIG_FILENAME);
    let temp_path = config_dir.join(format!(".{CONFIG_FILENAME}.tmp"));
    fs::write(&temp_path, text)?;
    fs::rename(&temp_path, &final_path)?;
    Ok(())
}
