#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use jiff::Timestamp;

use pimple_core::ical::expand::expand_in_range;
use pimple_core::ical::parse::parse_ics;
use pimple_core::{CollectionId, EventTime};

fn fixture(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ics")
        .join(name);
    std::fs::read_to_string(p).unwrap()
}

fn ts(s: &str) -> Timestamp {
    s.parse().unwrap()
}

#[test]
fn non_recurring_event_yields_one_instance_when_in_range() {
    let event = parse_ics(&fixture("utc.ics"), CollectionId::new("test")).unwrap();
    let instances = expand_in_range(
        &event,
        ts("2026-04-25T00:00:00Z"),
        ts("2026-04-26T00:00:00Z"),
    )
    .unwrap();
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].event_uid, "utc-1");
}

#[test]
fn non_recurring_event_yields_no_instance_when_out_of_range() {
    let event = parse_ics(&fixture("utc.ics"), CollectionId::new("test")).unwrap();
    let instances = expand_in_range(
        &event,
        ts("2027-01-01T00:00:00Z"),
        ts("2027-02-01T00:00:00Z"),
    )
    .unwrap();
    assert!(instances.is_empty());
}

#[test]
fn weekly_recurrence_yields_correct_instances_in_range() {
    let event = parse_ics(&fixture("weekly.ics"), CollectionId::new("test")).unwrap();
    let instances = expand_in_range(
        &event,
        ts("2026-01-01T00:00:00Z"),
        ts("2026-02-01T00:00:00Z"),
    )
    .unwrap();
    // Tuesdays in Jan 2026: 6, 13, 20, 27 → 4 instances.
    assert_eq!(instances.len(), 4);
}

#[test]
fn exdate_skips_the_excluded_instance() {
    let event = parse_ics(
        &fixture("weekly_with_exdate.ics"),
        CollectionId::new("test"),
    )
    .unwrap();
    let instances = expand_in_range(
        &event,
        ts("2026-01-01T00:00:00Z"),
        ts("2026-02-01T00:00:00Z"),
    )
    .unwrap();
    assert_eq!(instances.len(), 3);
    let starts: Vec<_> = instances
        .iter()
        .filter_map(|i| match &i.start {
            EventTime::Utc { instant } => Some(*instant),
            _ => None,
        })
        .collect();
    assert!(!starts.contains(&ts("2026-01-20T10:00:00Z")));
}

#[test]
fn override_replaces_master_instance_at_recurrence_id() {
    let event = parse_ics(
        &fixture("recurring_with_override.ics"),
        CollectionId::new("test"),
    )
    .unwrap();
    let instances = expand_in_range(
        &event,
        ts("2026-04-19T00:00:00Z"),
        ts("2026-04-26T00:00:00Z"),
    )
    .unwrap();
    let modified = instances
        .iter()
        .find(|i| i.is_override)
        .expect("at least one override instance in the week");
    assert_eq!(modified.summary, "Daily standup (moved 1h later)");
}
