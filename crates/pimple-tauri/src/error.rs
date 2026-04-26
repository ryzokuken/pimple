//! IPC error type that converts from `pimple_core::CoreError`.

use serde::Serialize;
use thiserror::Error;

#[expect(dead_code, reason = "IPC commands added in Task 13")]
#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IpcError {
    #[error("io: {message}")]
    Io { message: String },
    #[error("vdir: {message}")]
    Vdir { message: String },
    #[error("ical: {message}")]
    Ical { message: String },
    #[error("internal: {message}")]
    Internal { message: String },
}

impl From<pimple_core::CoreError> for IpcError {
    fn from(value: pimple_core::CoreError) -> Self {
        match value {
            pimple_core::CoreError::Io(e) => Self::Io { message: e.to_string() },
            pimple_core::CoreError::VdirLayout(m) => Self::Vdir { message: m },
            pimple_core::CoreError::Conversion(m) | pimple_core::CoreError::IcalParse(m) => {
                Self::Ical { message: m }
            }
        }
    }
}

#[expect(dead_code, reason = "IPC commands added in Task 13")]
pub type IpcResult<T> = std::result::Result<T, IpcError>;
