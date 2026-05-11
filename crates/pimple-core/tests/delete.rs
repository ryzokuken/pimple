//! Integration tests for `pimple_core::write::delete_event`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use pimple_core::event::{DeleteEventRequest, RecurringScope};
use pimple_core::ical::parse::parse_ics;
use pimple_core::{CollectionId, CoreError, EventTime};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn setup_collection(name: &str, ics_content: &str, ics_filename: &str) -> (TempDir, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let collection_dir = tmp.path().join(name);
    std::fs::create_dir_all(&collection_dir).unwrap();
    std::fs::write(collection_dir.join(ics_filename), ics_content).unwrap();
    (tmp, collection_dir)
}

fn hex_sha256(s: &str) -> String {
    hex::encode(Sha256::digest(s.as_bytes()))
}

const NON_RECURRING_ICS: &str = include_str!("fixtures/ics/utc.ics");
const WEEKLY_ICS: &str = include_str!("fixtures/ics/weekly.ics");
const RECURRING_WITH_OVERRIDE_ICS: &str = include_str!("fixtures/ics/recurring_with_override.ics");

fn uid_of(ics: &str) -> String {
    let event = parse_ics(ics, CollectionId::new("test")).unwrap();
    event.uid
}

// ─── All scope ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_all_removes_the_file() {
    let uid = uid_of(NON_RECURRING_ICS);
    let (_tmp, dir) = setup_collection("cal", NON_RECURRING_ICS, &format!("{uid}.ics"));
    let path = dir.join(format!("{uid}.ics"));
    assert!(path.exists());

    pimple_core::write::delete_event(
        &dir,
        &DeleteEventRequest {
            uid: uid.clone(),
            collection_id: CollectionId::new("cal"),
            expected_raw_hash: hex_sha256(NON_RECURRING_ICS),
            scope: RecurringScope::All,
            occurrence: None,
        },
    )
    .await
    .unwrap();

    assert!(!path.exists(), "file should be deleted");
}

#[tokio::test]
async fn delete_all_on_recurring_removes_the_file() {
    let uid = uid_of(WEEKLY_ICS);
    let (_tmp, dir) = setup_collection("cal", WEEKLY_ICS, &format!("{uid}.ics"));

    pimple_core::write::delete_event(
        &dir,
        &DeleteEventRequest {
            uid: uid.clone(),
            collection_id: CollectionId::new("cal"),
            expected_raw_hash: hex_sha256(WEEKLY_ICS),
            scope: RecurringScope::All,
            occurrence: None,
        },
    )
    .await
    .unwrap();

    assert!(!dir.join(format!("{uid}.ics")).exists());
}

// ─── ThisInstance scope ──────────────────────────────────────────────────────

#[tokio::test]
async fn delete_this_instance_adds_exdate_to_master() {
    let uid = uid_of(WEEKLY_ICS);
    let (_tmp, dir) = setup_collection("cal", WEEKLY_ICS, &format!("{uid}.ics"));
    let path = dir.join(format!("{uid}.ics"));

    // The weekly.ics fixture's DTSTART is in America/New_York.
    let occ = EventTime::Zoned {
        zoned: "2026-04-22T10:00:00-04:00[America/New_York]"
            .parse()
            .unwrap(),
    };

    pimple_core::write::delete_event(
        &dir,
        &DeleteEventRequest {
            uid: uid.clone(),
            collection_id: CollectionId::new("cal"),
            expected_raw_hash: hex_sha256(WEEKLY_ICS),
            scope: RecurringScope::ThisInstance,
            occurrence: Some(occ),
        },
    )
    .await
    .unwrap();

    let new_content = std::fs::read_to_string(&path).unwrap();
    assert!(new_content.contains("EXDATE;TZID=America/New_York:20260422T100000"));
    // The RRULE is preserved.
    assert!(new_content.contains("RRULE:FREQ=WEEKLY"));
}

#[tokio::test]
async fn delete_this_instance_without_occurrence_errors() {
    let uid = uid_of(WEEKLY_ICS);
    let (_tmp, dir) = setup_collection("cal", WEEKLY_ICS, &format!("{uid}.ics"));

    let err = pimple_core::write::delete_event(
        &dir,
        &DeleteEventRequest {
            uid: uid.clone(),
            collection_id: CollectionId::new("cal"),
            expected_raw_hash: hex_sha256(WEEKLY_ICS),
            scope: RecurringScope::ThisInstance,
            occurrence: None,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, CoreError::InvalidScope(_)));
}

// ─── ThisAndFuture scope ─────────────────────────────────────────────────────

#[tokio::test]
async fn delete_this_and_future_truncates_rrule_and_drops_later_overrides() {
    let uid = uid_of(RECURRING_WITH_OVERRIDE_ICS);
    let (_tmp, dir) = setup_collection("cal", RECURRING_WITH_OVERRIDE_ICS, &format!("{uid}.ics"));
    let path = dir.join(format!("{uid}.ics"));

    // Cut at the override's RECURRENCE-ID — it should be dropped.
    let occ = EventTime::Zoned {
        zoned: "2026-04-22T10:00:00-04:00[America/New_York]"
            .parse()
            .unwrap(),
    };

    pimple_core::write::delete_event(
        &dir,
        &DeleteEventRequest {
            uid: uid.clone(),
            collection_id: CollectionId::new("cal"),
            expected_raw_hash: hex_sha256(RECURRING_WITH_OVERRIDE_ICS),
            scope: RecurringScope::ThisAndFuture,
            occurrence: Some(occ),
        },
    )
    .await
    .unwrap();

    let new_content = std::fs::read_to_string(&path).unwrap();
    // UNTIL added in UTC form (the cutoff converted to absolute time).
    assert!(new_content.contains("UNTIL="));
    // Override block stripped.
    assert_eq!(new_content.matches("BEGIN:VEVENT").count(), 1);
    assert!(!new_content.contains("RECURRENCE-ID"));
}

// ─── Conflict detection ──────────────────────────────────────────────────────

#[tokio::test]
async fn delete_with_wrong_hash_returns_conflict_and_leaves_file_unchanged() {
    let uid = uid_of(NON_RECURRING_ICS);
    let (_tmp, dir) = setup_collection("cal", NON_RECURRING_ICS, &format!("{uid}.ics"));
    let path = dir.join(format!("{uid}.ics"));
    let original = std::fs::read_to_string(&path).unwrap();

    let err = pimple_core::write::delete_event(
        &dir,
        &DeleteEventRequest {
            uid: uid.clone(),
            collection_id: CollectionId::new("cal"),
            expected_raw_hash: "0".repeat(64),
            scope: RecurringScope::All,
            occurrence: None,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, CoreError::Conflict { .. }));
    // File untouched.
    assert!(path.exists());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
}
