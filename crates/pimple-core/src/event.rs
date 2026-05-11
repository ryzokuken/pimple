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
    /// SHA-256 of `raw_ics` as a 64-char lowercase hex string. Used for
    /// optimistic concurrency on update/delete; see v0.2 spec §4.1.
    pub raw_hash: String,
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
    /// `RECURRENCE-ID` value for this concrete occurrence (the master's
    /// DTSTART for non-recurring events; the occurrence's original start
    /// time for recurring ones, even when overridden). The frontend uses
    /// this to address "this instance" on delete/edit operations.
    pub recurrence_id: EventTime,
    /// Master event's `raw_hash`. Carried so the frontend can pass it back
    /// to update/delete commands for concurrent-write detection.
    pub raw_hash: String,
    /// Whether the underlying `Event` has any `RRULE` — drives the UI's
    /// "show recurring-scope dialog?" decision without a separate fetch.
    pub is_recurring: bool,
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

/// Which recurring scope a delete or edit applies to. See v0.2 spec §4.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../frontend/src/lib/ipc/types/")]
#[serde(rename_all = "snake_case")]
pub enum RecurringScope {
    /// Just the picked instance. Implemented via EXDATE (delete) or RECURRENCE-ID
    /// override (edit).
    ThisInstance,
    /// The picked instance plus every later occurrence. Implemented via
    /// truncating the master RRULE with `UNTIL` plus dropping overrides
    /// at-or-after the cutoff. For edit, a new continuation file is also written.
    ThisAndFuture,
    /// The entire series. For delete, removes the file. For edit, mutates the
    /// master VEVENT in place.
    All,
}

/// User-supplied data for deleting an event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../frontend/src/lib/ipc/types/")]
pub struct DeleteEventRequest {
    pub uid: String,
    pub collection_id: CollectionId,
    /// SHA-256 hex of `raw_ics` as last seen by the frontend. The backend
    /// re-hashes the on-disk file before mutating and aborts with
    /// `CoreError::Conflict` on mismatch.
    pub expected_raw_hash: String,
    pub scope: RecurringScope,
    /// For `ThisInstance` and `ThisAndFuture`, identifies which occurrence.
    /// Ignored for `All`. Should equal the occurrence's `RECURRENCE-ID`.
    pub occurrence: Option<EventTime>,
}
