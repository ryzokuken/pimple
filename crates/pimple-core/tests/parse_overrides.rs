#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use pimple_core::CollectionId;
use pimple_core::ical::parse::parse_ics;

fn read_fixture(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ics")
        .join(name);
    std::fs::read_to_string(p).unwrap()
}

#[test]
fn collapses_master_and_override_into_one_event() {
    let ics = read_fixture("recurring_with_override.ics");
    let event = parse_ics(&ics, CollectionId::new("test")).unwrap();

    assert_eq!(event.uid, "standup-1");
    assert_eq!(event.summary, "Daily standup");
    assert!(event.rrule.is_some());
    assert_eq!(event.overrides.len(), 1);
    assert_eq!(
        event.overrides[0].summary.as_deref(),
        Some("Daily standup (moved 1h later)")
    );
}

#[test]
fn rejects_two_master_vevents_in_one_file() {
    let ics = "BEGIN:VCALENDAR\nVERSION:2.0\nPRODID:-//pimple//tests//EN\n\
        BEGIN:VEVENT\nUID:a\nDTSTAMP:20260101T000000Z\nDTSTART:20260101T000000Z\n\
        DTEND:20260101T010000Z\nSUMMARY:A\nEND:VEVENT\n\
        BEGIN:VEVENT\nUID:b\nDTSTAMP:20260101T000000Z\nDTSTART:20260101T000000Z\n\
        DTEND:20260101T010000Z\nSUMMARY:B\nEND:VEVENT\n\
        END:VCALENDAR\n";
    assert!(parse_ics(ics, CollectionId::new("test")).is_err());
}
