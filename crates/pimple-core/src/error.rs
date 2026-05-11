//! Error type for `pimple-core`.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("vdir layout: {0}")]
    VdirLayout(String),
    #[error("datetime conversion: {0}")]
    Conversion(String),
    #[error("ical parse: {0}")]
    IcalParse(String),
    /// Optimistic-concurrency check failed: the on-disk `raw_ics` hash drifted
    /// from the one the caller passed. The user must reload and retry.
    #[error("conflict: event {uid} was modified externally")]
    Conflict { uid: String },
    /// A delete/edit request named a scope inconsistent with the event's
    /// structure (e.g., `ThisInstance` on a non-recurring event).
    #[error("invalid scope: {0}")]
    InvalidScope(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
