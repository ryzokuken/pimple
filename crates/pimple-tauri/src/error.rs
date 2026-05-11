//! IPC error type that converts from `pimple_core::CoreError`.

use serde::Serialize;
use thiserror::Error;

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
    /// Optimistic-concurrency mismatch on update/delete. The frontend should
    /// reload the event and prompt the user.
    #[error("conflict: event {uid} was modified externally")]
    Conflict { uid: String },
    /// Caller passed a recurring scope inconsistent with the event's shape.
    #[error("invalid scope: {message}")]
    InvalidScope { message: String },
}

impl From<pimple_core::CoreError> for IpcError {
    fn from(value: pimple_core::CoreError) -> Self {
        match value {
            pimple_core::CoreError::Io(e) => Self::Io {
                message: e.to_string(),
            },
            pimple_core::CoreError::VdirLayout(m) => Self::Vdir { message: m },
            pimple_core::CoreError::Conversion(m) | pimple_core::CoreError::IcalParse(m) => {
                Self::Ical { message: m }
            }
            pimple_core::CoreError::Conflict { uid } => Self::Conflict { uid },
            pimple_core::CoreError::InvalidScope(m) => Self::InvalidScope { message: m },
        }
    }
}

pub type IpcResult<T> = std::result::Result<T, IpcError>;
