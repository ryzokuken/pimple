//! Event types.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::CollectionId;
use crate::EventTime;

/// Persisted event definition. One per iCalendar UID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../frontend/src/lib/ipc/types/")]
pub struct Event {
    pub uid: String,
    pub collection_id: CollectionId,
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start: EventTime,
    pub end: EventTime,
    pub rrule: Option<RRuleSpec>,
    pub exdates: Vec<EventTime>,
    pub overrides: Vec<OverrideInstance>,
    #[ts(type = "string")]
    pub created_at: Timestamp,
    #[ts(type = "string")]
    pub modified_at: Timestamp,
    /// Original `.ics` content. Preserved for round-trip on future edit.
    #[ts(skip)]
    pub raw_ics: String,
}

/// Verbatim `RRULE` line (everything after `RRULE:`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../frontend/src/lib/ipc/types/")]
pub struct RRuleSpec {
    pub line: String,
}

/// A single override of a recurring event (a VEVENT with a `RECURRENCE-ID`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../frontend/src/lib/ipc/types/")]
pub struct OverrideInstance {
    pub recurrence_id: EventTime,
    pub start: EventTime,
    pub end: EventTime,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
}

/// A single concrete occurrence. Derived on query; never persisted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../frontend/src/lib/ipc/types/")]
pub struct EventInstance {
    pub event_uid: String,
    pub collection_id: CollectionId,
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start: EventTime,
    pub end: EventTime,
    pub is_override: bool,
}

/// User-supplied data for creating a new event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../frontend/src/lib/ipc/types/")]
pub struct CreateEventRequest {
    pub collection_id: CollectionId,
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start: EventTime,
    pub end: EventTime,
    pub rrule: Option<RRuleSpec>,
}
