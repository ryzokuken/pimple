//! pimple-core: vdir-backed calendar engine.

#![forbid(unsafe_code)]

pub mod collection;
pub mod config;
pub mod index;
pub mod error;
pub mod event;
pub mod ical;
pub mod id;
pub mod time;
pub mod vdir;
pub mod watcher;
pub mod write;

pub use collection::Collection;
pub use config::{AppConfig, WeekStart};
pub use error::{CoreError, Result};
pub use event::{CreateEventRequest, Event, EventInstance, OverrideInstance, RRuleSpec};
pub use id::CollectionId;
pub use time::EventTime;
