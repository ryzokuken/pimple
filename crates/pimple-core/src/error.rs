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
}

pub type Result<T> = std::result::Result<T, CoreError>;
