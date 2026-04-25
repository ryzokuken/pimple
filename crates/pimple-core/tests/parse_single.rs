#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use pimple_core::ical::parse::parse_ics;
use pimple_core::{CollectionId, EventTime};

fn read_fixture(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ics")
        .join(name);
    std::fs::read_to_string(p).unwrap()
}

fn cid() -> CollectionId {
    CollectionId::new("test")
}

#[test]
fn parses_all_day_event() {
    let ics = read_fixture("all_day.ics");
    let event = parse_ics(&ics, cid()).unwrap();
    assert_eq!(event.uid, "all-day-1");
    assert_eq!(event.summary, "All-day event");
    matches!(event.start, EventTime::AllDay { .. });
}

#[test]
fn parses_floating_event() {
    let ics = read_fixture("floating.ics");
    let event = parse_ics(&ics, cid()).unwrap();
    matches!(event.start, EventTime::Floating { .. });
}

#[test]
fn parses_utc_event() {
    let ics = read_fixture("utc.ics");
    let event = parse_ics(&ics, cid()).unwrap();
    let EventTime::Utc { instant } = event.start else {
        panic!("expected UTC event time");
    };
    // 2026-04-25T15:30:00Z = Unix second 1777131000
    assert_eq!(instant.as_second(), 1_777_131_000);
}

#[test]
fn parses_zoned_event() {
    let ics = read_fixture("zoned.ics");
    let event = parse_ics(&ics, cid()).unwrap();
    let EventTime::Zoned { zoned } = event.start else {
        panic!("expected zoned event time");
    };
    assert_eq!(zoned.time_zone().iana_name(), Some("America/New_York"));
}

#[test]
fn preserves_location_and_description() {
    let ics = read_fixture("with_location_and_description.ics");
    let event = parse_ics(&ics, cid()).unwrap();
    assert_eq!(event.location.as_deref(), Some("Café Nord, Berlin"));
    assert_eq!(
        event.description.as_deref(),
        Some("Catch-up about Q2 planning.\nBring the deck.")
    );
}

#[test]
fn raw_ics_is_preserved_verbatim() {
    let ics = read_fixture("utc.ics");
    let event = parse_ics(&ics, cid()).unwrap();
    assert_eq!(event.raw_ics, ics);
}
