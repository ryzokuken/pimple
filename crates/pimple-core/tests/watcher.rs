#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::Duration;

use jiff::Timestamp;
use tempfile::TempDir;

use pimple_core::CollectionId;
use pimple_core::index::EventIndex;
use pimple_core::watcher::FilesystemWatcher;

fn ts(s: &str) -> Timestamp {
    s.parse().unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn detects_new_ics_file_and_indexes_it() {
    let tmp = TempDir::new().unwrap();
    let collection_dir = tmp.path().join("personal");
    std::fs::create_dir_all(&collection_dir).unwrap();

    let index = EventIndex::new();
    let _watcher = FilesystemWatcher::start(tmp.path().to_path_buf(), index.clone())
        .await
        .unwrap();

    // Give the watcher a moment to settle.
    tokio::time::sleep(Duration::from_millis(150)).await;

    let ics = "BEGIN:VCALENDAR\nVERSION:2.0\nPRODID:-//pimple//tests//EN\n\
        BEGIN:VEVENT\nUID:fs-1\nDTSTAMP:20260101T000000Z\n\
        DTSTART:20260425T100000Z\nDTEND:20260425T110000Z\nSUMMARY:From FS\n\
        END:VEVENT\nEND:VCALENDAR\n";
    std::fs::write(collection_dir.join("fs-1.ics"), ics).unwrap();

    // Wait through the debounce window plus a margin.
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
            assert_eq!(out[0].event_uid, "fs-1");
            return;
        }
    }
    panic!("watcher did not pick up the new .ics within 2s");
}
