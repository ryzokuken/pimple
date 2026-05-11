//! pimple-core: vdir-backed calendar engine.

#![forbid(unsafe_code)]

pub mod collection;
pub mod config;
pub mod config_store;
pub mod error;
pub mod event;
pub mod ical;
pub mod id;
pub mod index;
pub mod time;
pub mod vdir;
pub mod watcher;
pub mod write;

pub use collection::Collection;
pub use config::{AppConfig, WeekStart, WindowGeometry};
pub use error::{CoreError, Result};
pub use event::{
    CreateEventRequest, DeleteEventRequest, Event, EventInstance, OverrideInstance, RRuleSpec,
    RecurringScope,
};
pub use id::CollectionId;
pub use time::EventTime;
