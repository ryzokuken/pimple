#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use jiff::Timestamp;

use pimple_core::CollectionId;
use pimple_core::ical::parse::parse_ics;
use pimple_core::index::{EventIndex, IndexChange};

fn fixture(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ics")
        .join(name);
    std::fs::read_to_string(p).unwrap()
}

fn ts(s: &str) -> Timestamp {
    s.parse().unwrap()
}

#[tokio::test]
async fn empty_index_returns_no_instances() {
    let idx = EventIndex::new();
    let out = idx
        .events_in_range(ts("2026-01-01T00:00:00Z"), ts("2026-12-31T00:00:00Z"), &[])
        .await
        .unwrap();
    assert!(out.is_empty());
}

#[tokio::test]
async fn inserted_event_is_returned_when_in_range_and_collection_visible() {
    let idx = EventIndex::new();
    let event = parse_ics(&fixture("utc.ics"), CollectionId::new("personal")).unwrap();
    idx.apply_change(IndexChange::upsert(event)).await;
    let out = idx
        .events_in_range(
            ts("2026-04-25T00:00:00Z"),
            ts("2026-04-26T00:00:00Z"),
            &[CollectionId::new("personal")],
        )
        .await
        .unwrap();
    assert_eq!(out.len(), 1);
}

#[tokio::test]
async fn invisible_collection_filters_out_events() {
    let idx = EventIndex::new();
    let event = parse_ics(&fixture("utc.ics"), CollectionId::new("personal")).unwrap();
    idx.apply_change(IndexChange::upsert(event)).await;
    let out = idx
        .events_in_range(
            ts("2026-04-25T00:00:00Z"),
            ts("2026-04-26T00:00:00Z"),
            &[CollectionId::new("work")], // personal is not in the visible set
        )
        .await
        .unwrap();
    assert!(out.is_empty());
}

#[tokio::test]
async fn subscribe_receives_change_events() {
    let idx = EventIndex::new();
    let mut rx = idx.subscribe();

    let event = parse_ics(&fixture("utc.ics"), CollectionId::new("personal")).unwrap();
    idx.apply_change(IndexChange::upsert(event)).await;

    let received = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .expect("receive within 1s")
        .expect("channel still open");
    match received {
        IndexChange::Upsert { uid, .. } => assert_eq!(uid, "utc-1"),
        other => panic!("unexpected change: {other:?}"),
    }
}

#[tokio::test]
async fn remove_drops_the_event() {
    let idx = EventIndex::new();
    let event = parse_ics(&fixture("utc.ics"), CollectionId::new("personal")).unwrap();
    idx.apply_change(IndexChange::upsert(event)).await;
    idx.apply_change(IndexChange::Remove {
        uid: "utc-1".into(),
    })
    .await;

    let out = idx
        .events_in_range(
            ts("2026-04-25T00:00:00Z"),
            ts("2026-04-26T00:00:00Z"),
            &[CollectionId::new("personal")],
        )
        .await
        .unwrap();
    assert!(out.is_empty());
}
