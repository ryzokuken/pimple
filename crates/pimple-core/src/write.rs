//! Write path: create / delete `.ics` files atomically inside a collection directory.

use std::path::Path;

use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::error::{CoreError, Result};
use crate::event::{CreateEventRequest, DeleteEventRequest, RecurringScope};
use crate::ical::build::build_ics;
use crate::ical::patch::{IcsMutation, patch_ics};

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
    write_atomic(&final_path, ics.as_bytes()).await?;

    Ok(uid)
}

/// Delete an event, honouring [`RecurringScope`].
///
/// * `RecurringScope::All` removes the underlying file.
/// * `RecurringScope::ThisInstance` appends an `EXDATE` to the master and
///   re-writes the file.
/// * `RecurringScope::ThisAndFuture` truncates the master `RRULE` with
///   `UNTIL=<occurrence>` and drops any override `RECURRENCE-ID`s
///   at-or-after the occurrence.
///
/// All paths verify `expected_raw_hash` matches the on-disk file's SHA-256
/// before mutating, returning [`CoreError::Conflict`] on mismatch. Writes
/// are atomic via temp-file + rename.
///
/// # Errors
///
/// * [`CoreError::Conflict`] — on-disk hash drift since the caller's read.
/// * [`CoreError::InvalidScope`] — `ThisInstance`/`ThisAndFuture` without
///   an `occurrence` value.
/// * [`CoreError::Io`] — file I/O failures.
/// * [`CoreError::IcalParse`] — malformed on-disk `.ics`.
pub async fn delete_event(collection_path: &Path, req: &DeleteEventRequest) -> Result<()> {
    if !collection_path.is_dir() {
        return Err(CoreError::VdirLayout(format!(
            "collection directory {} does not exist",
            collection_path.display()
        )));
    }
    let file_path = collection_path.join(format!("{}.ics", req.uid));
    let raw = fs::read_to_string(&file_path).await?;
    let actual_hash = hex_sha256(raw.as_bytes());
    if actual_hash != req.expected_raw_hash {
        return Err(CoreError::Conflict {
            uid: req.uid.clone(),
        });
    }

    match req.scope {
        RecurringScope::All => {
            let trash = collection_path.join(format!(".{}.ics.deleting", req.uid));
            fs::rename(&file_path, &trash).await?;
            fs::remove_file(&trash).await?;
        }
        RecurringScope::ThisInstance => {
            let occ = req.occurrence.as_ref().ok_or_else(|| {
                CoreError::InvalidScope("ThisInstance requires an occurrence recurrence_id".into())
            })?;
            let patched = patch_ics(&raw, &[IcsMutation::AddExdate(occ.clone())])?;
            write_atomic(&file_path, patched.as_bytes()).await?;
        }
        RecurringScope::ThisAndFuture => {
            let occ = req.occurrence.as_ref().ok_or_else(|| {
                CoreError::InvalidScope("ThisAndFuture requires an occurrence recurrence_id".into())
            })?;
            let mutations = [
                IcsMutation::TruncateRRuleUntil(occ.clone()),
                IcsMutation::RemoveOverridesAtOrAfter(occ.clone()),
            ];
            let patched = patch_ics(&raw, &mutations)?;
            write_atomic(&file_path, patched.as_bytes()).await?;
        }
    }
    Ok(())
}

fn hex_sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

async fn write_atomic(final_path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = final_path.parent().ok_or_else(|| {
        CoreError::VdirLayout(format!("no parent directory for {}", final_path.display()))
    })?;
    let file_name = final_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            CoreError::VdirLayout(format!("non-utf8 filename: {}", final_path.display()))
        })?;
    let temp_path = parent.join(format!(".{file_name}.tmp"));
    {
        let mut f = fs::File::create(&temp_path).await?;
        f.write_all(bytes).await?;
        f.sync_all().await?;
    }
    fs::rename(&temp_path, final_path).await?;
    Ok(())
}
