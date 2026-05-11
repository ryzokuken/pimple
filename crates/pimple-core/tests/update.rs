//! Integration tests for `pimple_core::write::update_event`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use pimple_core::event::{RRuleSpec, RecurringScope, UpdateEventRequest};
use pimple_core::ical::parse::parse_ics;
use pimple_core::{CollectionId, CoreError, EventTime};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn setup_collection(name: &str, ics_content: &str, ics_filename: &str) -> (TempDir, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join(name);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(ics_filename), ics_content).unwrap();
    (tmp, dir)
}

fn hex_sha256(s: &str) -> String {
    hex::encode(Sha256::digest(s.as_bytes()))
}

fn uid_of(ics: &str) -> String {
    parse_ics(ics, CollectionId::new("test")).unwrap().uid
}

const NON_RECURRING_ICS: &str = include_str!("fixtures/ics/utc.ics");
const WEEKLY_ICS: &str = include_str!("fixtures/ics/weekly.ics");
const WITH_VALARM_ICS: &str = include_str!("fixtures/ics/with_valarm.ics");

fn base_request(uid: &str, hash: &str, scope: RecurringScope) -> UpdateEventRequest {
    UpdateEventRequest {
        uid: uid.to_owned(),
        collection_id: CollectionId::new("cal"),
        expected_raw_hash: hash.to_owned(),
        scope,
        occurrence: None,
        summary: "Renamed".to_owned(),
        description: Some("New description".to_owned()),
        location: Some("New room".to_owned()),
        start: EventTime::Utc {
            instant: "2026-04-25T10:00:00Z".parse().unwrap(),
        },
        end: EventTime::Utc {
            instant: "2026-04-25T11:00:00Z".parse().unwrap(),
        },
        rrule: None,
    }
}

// ─── All scope ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn update_all_rewrites_master_fields_in_place() {
    let uid = uid_of(NON_RECURRING_ICS);
    let (_tmp, dir) = setup_collection("cal", NON_RECURRING_ICS, &format!("{uid}.ics"));
    let path = dir.join(format!("{uid}.ics"));

    let res = pimple_core::write::update_event(
        &dir,
        &base_request(&uid, &hex_sha256(NON_RECURRING_ICS), RecurringScope::All),
    )
    .await
    .unwrap();
    assert!(res.is_none(), "in-place updates return no new UID");

    let new = std::fs::read_to_string(&path).unwrap();
    assert!(new.contains("SUMMARY:Renamed"));
    assert!(new.contains("DESCRIPTION:New description"));
    assert!(new.contains("LOCATION:New room"));
    // UID and DTSTAMP survive.
    assert!(new.contains(&format!("UID:{uid}")));
}

#[tokio::test]
async fn update_all_preserves_valarm_and_x_vendor_properties() {
    let uid = uid_of(WITH_VALARM_ICS);
    let (_tmp, dir) = setup_collection("cal", WITH_VALARM_ICS, &format!("{uid}.ics"));
    let path = dir.join(format!("{uid}.ics"));

    pimple_core::write::update_event(
        &dir,
        &UpdateEventRequest {
            rrule: Some(RRuleSpec {
                line: "FREQ=DAILY;COUNT=5".into(),
            }),
            ..base_request(&uid, &hex_sha256(WITH_VALARM_ICS), RecurringScope::All)
        },
    )
    .await
    .unwrap();

    let new = std::fs::read_to_string(&path).unwrap();
    assert!(new.contains("BEGIN:VALARM"));
    assert!(new.contains("ACTION:DISPLAY"));
    assert!(new.contains("X-APPLE-CALENDAR-COLOR:#FF5733"));
    assert!(new.contains("RRULE:FREQ=DAILY;COUNT=5"));
}

// ─── ThisInstance scope ──────────────────────────────────────────────────────

#[tokio::test]
async fn update_this_instance_appends_override_vevent() {
    let uid = uid_of(WEEKLY_ICS);
    let (_tmp, dir) = setup_collection("cal", WEEKLY_ICS, &format!("{uid}.ics"));
    let path = dir.join(format!("{uid}.ics"));

    let occ = EventTime::Zoned {
        zoned: "2026-04-22T10:00:00-04:00[America/New_York]"
            .parse()
            .unwrap(),
    };
    let new_start = EventTime::Zoned {
        zoned: "2026-04-22T11:00:00-04:00[America/New_York]"
            .parse()
            .unwrap(),
    };
    let new_end = EventTime::Zoned {
        zoned: "2026-04-22T11:30:00-04:00[America/New_York]"
            .parse()
            .unwrap(),
    };

    pimple_core::write::update_event(
        &dir,
        &UpdateEventRequest {
            summary: "Standup (moved 1h)".into(),
            start: new_start,
            end: new_end,
            occurrence: Some(occ),
            ..base_request(&uid, &hex_sha256(WEEKLY_ICS), RecurringScope::ThisInstance)
        },
    )
    .await
    .unwrap();

    let new = std::fs::read_to_string(&path).unwrap();
    assert_eq!(new.matches("BEGIN:VEVENT").count(), 2);
    assert!(new.contains("RECURRENCE-ID;TZID=America/New_York:20260422T100000"));
    assert!(new.contains("DTSTART;TZID=America/New_York:20260422T110000"));
    assert!(new.contains("SUMMARY:Standup (moved 1h)"));
    // The master RRULE is untouched.
    assert!(new.contains("RRULE:FREQ=WEEKLY"));
}

#[tokio::test]
async fn update_this_instance_without_occurrence_errors() {
    let uid = uid_of(WEEKLY_ICS);
    let (_tmp, dir) = setup_collection("cal", WEEKLY_ICS, &format!("{uid}.ics"));

    let err = pimple_core::write::update_event(
        &dir,
        &base_request(&uid, &hex_sha256(WEEKLY_ICS), RecurringScope::ThisInstance),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, CoreError::InvalidScope(_)));
}

// ─── ThisAndFuture scope ─────────────────────────────────────────────────────

#[tokio::test]
async fn update_this_and_future_splits_master_and_writes_continuation() {
    let uid = uid_of(WEEKLY_ICS);
    let (_tmp, dir) = setup_collection("cal", WEEKLY_ICS, &format!("{uid}.ics"));
    let original_path = dir.join(format!("{uid}.ics"));

    let occ = EventTime::Zoned {
        zoned: "2026-04-22T10:00:00-04:00[America/New_York]"
            .parse()
            .unwrap(),
    };
    let new_start = EventTime::Zoned {
        zoned: "2026-04-22T11:00:00-04:00[America/New_York]"
            .parse()
            .unwrap(),
    };
    let new_end = EventTime::Zoned {
        zoned: "2026-04-22T11:30:00-04:00[America/New_York]"
            .parse()
            .unwrap(),
    };

    let new_uid = pimple_core::write::update_event(
        &dir,
        &UpdateEventRequest {
            summary: "Standup (new pattern)".into(),
            start: new_start,
            end: new_end,
            occurrence: Some(occ),
            rrule: Some(RRuleSpec {
                line: "FREQ=WEEKLY".into(),
            }),
            ..base_request(&uid, &hex_sha256(WEEKLY_ICS), RecurringScope::ThisAndFuture)
        },
    )
    .await
    .unwrap()
    .expect("ThisAndFuture returns a new UID");

    assert_ne!(new_uid, uid, "continuation must get a fresh UID");

    // Master file is truncated.
    let master = std::fs::read_to_string(&original_path).unwrap();
    assert!(master.contains("UNTIL="));
    assert!(master.contains(&format!("UID:{uid}")));

    // Continuation file exists with new UID and new fields.
    let cont_path = dir.join(format!("{new_uid}.ics"));
    assert!(cont_path.exists());
    let cont = std::fs::read_to_string(&cont_path).unwrap();
    assert!(cont.contains(&format!("UID:{new_uid}")));
    assert!(cont.contains("SUMMARY:Standup (new pattern)"));
    assert!(cont.contains("DTSTART;TZID=America/New_York:20260422T110000"));
    assert!(cont.contains("RRULE:FREQ=WEEKLY"));
}

#[tokio::test]
async fn update_this_and_future_without_occurrence_errors() {
    let uid = uid_of(WEEKLY_ICS);
    let (_tmp, dir) = setup_collection("cal", WEEKLY_ICS, &format!("{uid}.ics"));

    let err = pimple_core::write::update_event(
        &dir,
        &base_request(&uid, &hex_sha256(WEEKLY_ICS), RecurringScope::ThisAndFuture),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, CoreError::InvalidScope(_)));
}

// ─── Conflict detection ──────────────────────────────────────────────────────

#[tokio::test]
async fn update_with_wrong_hash_returns_conflict_and_leaves_file_unchanged() {
    let uid = uid_of(NON_RECURRING_ICS);
    let (_tmp, dir) = setup_collection("cal", NON_RECURRING_ICS, &format!("{uid}.ics"));
    let path = dir.join(format!("{uid}.ics"));
    let original = std::fs::read_to_string(&path).unwrap();

    let err = pimple_core::write::update_event(
        &dir,
        &base_request(&uid, &"0".repeat(64), RecurringScope::All),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, CoreError::Conflict { .. }));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
}
