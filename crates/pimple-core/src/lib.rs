//! pimple-core: vdir-backed calendar engine.

#![forbid(unsafe_code)]

pub mod collection;
pub mod config;
pub mod error;
pub mod event;
pub mod id;
pub mod time;
pub mod vdir;

pub use collection::Collection;
pub use config::{AppConfig, WeekStart};
pub use error::{CoreError, Result};
pub use event::{Event, EventInstance, OverrideInstance, RRuleSpec};
pub use id::CollectionId;
pub use time::EventTime;
