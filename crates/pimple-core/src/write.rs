//! Write path: create a new `.ics` file atomically inside a collection directory.

use std::path::Path;

use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::error::{CoreError, Result};
use crate::event::CreateEventRequest;
use crate::ical::build::build_ics;

/// Create a new event in the given collection directory and return its UID.
///
/// The write is atomic: the `.ics` is materialized via a temp file in the same
/// directory, then renamed into place. Concurrent readers (including pimsync)
/// never observe a partial file.
///
/// # Errors
///
/// Returns an error if the collection directory does not exist, or if the
/// underlying file I/O operations fail.
pub async fn create_event(collection_path: &Path, req: &CreateEventRequest) -> Result<String> {
    if !collection_path.is_dir() {
        return Err(CoreError::VdirLayout(format!(
            "collection directory {} does not exist",
            collection_path.display()
        )));
    }
    let (ics, uid) = build_ics(req)?;

    let final_path = collection_path.join(format!("{uid}.ics"));
    let temp_path = collection_path.join(format!(".{uid}.ics.tmp"));

    {
        let mut f = fs::File::create(&temp_path).await?;
        f.write_all(ics.as_bytes()).await?;
        f.sync_all().await?;
    }
    fs::rename(&temp_path, &final_path).await?;

    Ok(uid)
}
