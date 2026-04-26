#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::Duration;

use jiff::Timestamp;
use tempfile::TempDir;

use pimple_core::index::EventIndex;
use pimple_core::watcher::FilesystemWatcher;
use pimple_core::write::create_event;
use pimple_core::{CollectionId, CreateEventRequest, EventTime};

fn ts(s: &str) -> Timestamp {
    s.parse().unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn create_event_writes_atomically_and_watcher_picks_it_up() {
    let tmp = TempDir::new().unwrap();
    let collection_path = tmp.path().join("personal");
    std::fs::create_dir_all(&collection_path).unwrap();

    let index = EventIndex::new();
    let _watcher = FilesystemWatcher::start(tmp.path().to_path_buf(), index.clone())
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(150)).await;

    let req = CreateEventRequest {
        collection_id: CollectionId::new("personal"),
        summary: "Coffee".into(),
        description: None,
        location: None,
        start: EventTime::Utc {
            instant: "2026-04-25T15:30:00Z".parse().unwrap(),
        },
        end: EventTime::Utc {
            instant: "2026-04-25T16:30:00Z".parse().unwrap(),
        },
        rrule: None,
    };

    let uid = create_event(&collection_path, &req).await.unwrap();
    let written = collection_path.join(format!("{uid}.ics"));
    assert!(written.exists(), "the .ics file should exist");
    let written_text = std::fs::read_to_string(&written).unwrap();
    assert!(written_text.contains(&format!("UID:{uid}")));

    // Eventually the watcher reflects it.
    for _ in 0..40 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let out = index
            .events_in_range(
                ts("2026-04-25T00:00:00Z"),
                ts("2026-04-26T00:00:00Z"),
                &[CollectionId::new("personal")],
            )
            .await
            .unwrap();
        if !out.is_empty() {
            assert_eq!(out[0].event_uid, uid);
            return;
        }
    }
    panic!("watcher did not see the newly-created event within 2s");
}

#[tokio::test]
async fn create_event_rejects_nonexistent_collection() {
    let tmp = TempDir::new().unwrap();
    let nonexistent = tmp.path().join("does_not_exist");
    let req = CreateEventRequest {
        collection_id: CollectionId::new("does_not_exist"),
        summary: "x".into(),
        description: None,
        location: None,
        start: EventTime::Utc {
            instant: "2026-04-25T15:30:00Z".parse().unwrap(),
        },
        end: EventTime::Utc {
            instant: "2026-04-25T16:30:00Z".parse().unwrap(),
        },
        rrule: None,
    };
    assert!(create_event(&nonexistent, &req).await.is_err());
}
