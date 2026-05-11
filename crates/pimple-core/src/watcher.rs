//! Filesystem watcher that translates raw FS events into `IndexChange`s.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use notify::event::{Event as NotifyEvent, EventKind};
use notify::{RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, error, warn};

use crate::collection::Collection;
use crate::error::{CoreError, Result};
use crate::ical::parse::parse_ics;
use crate::index::{EventIndex, IndexChange};
use crate::vdir::layout::enumerate_collections;

/// Time window during which repeated events on the same path are coalesced.
const DEBOUNCE: Duration = Duration::from_millis(150);

pub struct FilesystemWatcher {
    /// Held to keep the watcher alive; dropped on shutdown.
    _watcher: RecommendedWatcher,
    /// Background task that processes debounced events.
    _processor: JoinHandle<()>,
}

impl FilesystemWatcher {
    /// Start watching `root`. The watcher does an initial full scan, then
    /// streams subsequent changes into `index`.
    ///
    /// # Errors
    ///
    /// Returns `CoreError` if the initial scan fails or if `notify` cannot
    /// set up the filesystem watch.
    pub async fn start(root: PathBuf, index: EventIndex) -> Result<Self> {
        initial_scan(&root, &index).await?;

        let (tx, rx) = mpsc::unbounded_channel::<NotifyEvent>();
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res| match res {
            Ok(event) => {
                if tx.send(event).is_err() {
                    debug!("watcher channel closed");
                }
            }
            Err(e) => warn!("notify error: {e}"),
        })
        .map_err(|e| CoreError::VdirLayout(format!("notify: {e}")))?;
        watcher
            .watch(&root, RecursiveMode::Recursive)
            .map_err(|e| CoreError::VdirLayout(format!("notify watch: {e}")))?;

        let processor = tokio::spawn(process_loop(root, index, rx));

        Ok(Self {
            _watcher: watcher,
            _processor: processor,
        })
    }
}

async fn initial_scan(root: &Path, index: &EventIndex) -> Result<()> {
    let collections = enumerate_collections(root)?;
    let mut events = Vec::new();
    for col in &collections {
        let entries = match std::fs::read_dir(&col.path) {
            Ok(e) => e,
            Err(e) => {
                warn!("read collection {}: {e}", col.path.display());
                continue;
            }
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) != Some("ics") {
                continue;
            }
            match std::fs::read_to_string(&p) {
                Ok(text) => match parse_ics(&text, col.id.clone()) {
                    Ok(event) => events.push(event),
                    Err(e) => warn!("parse {} failed: {e}", p.display()),
                },
                Err(e) => warn!("read {}: {e}", p.display()),
            }
        }
    }
    index.apply_change(IndexChange::FullReload { events }).await;
    Ok(())
}

async fn process_loop(
    root: PathBuf,
    index: EventIndex,
    mut rx: mpsc::UnboundedReceiver<NotifyEvent>,
) {
    let mut pending: HashMap<PathBuf, std::time::Instant> = HashMap::new();
    let collections: Arc<tokio::sync::RwLock<Vec<Collection>>> = Arc::new(
        tokio::sync::RwLock::new(enumerate_collections(&root).unwrap_or_default()),
    );

    loop {
        tokio::select! {
            event = rx.recv() => {
                let Some(event) = event else { break; };
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        let now = std::time::Instant::now();
                        for path in event.paths {
                            // If a collection directory itself was created/removed, schedule
                            // a full reload.
                            if path == root
                                || (path.parent() == Some(root.as_path()) && path.is_dir())
                            {
                                pending.insert(root.clone(), now);
                            } else {
                                pending.insert(path, now);
                            }
                        }
                    }
                    _ => {}
                }
            }
            () = tokio::time::sleep(DEBOUNCE) => {
                if pending.is_empty() {
                    continue;
                }
                let now = std::time::Instant::now();
                let mut to_process = Vec::new();
                pending.retain(|p, t| {
                    if now.duration_since(*t) >= DEBOUNCE {
                        to_process.push(p.clone());
                        false
                    } else {
                        true
                    }
                });
                for path in to_process {
                    if path == root {
                        let mut snapshot = collections.write().await;
                        *snapshot = enumerate_collections(&root).unwrap_or_default();
                        drop(snapshot);
                        if let Err(e) = full_reload(&root, &index).await {
                            error!("full reload failed: {e}");
                        }
                    } else if let Err(e) = handle_path(&path, &collections, &index).await {
                        error!("handle {} failed: {e}", path.display());
                    }
                }
            }
        }
    }
}

async fn full_reload(root: &Path, index: &EventIndex) -> Result<()> {
    initial_scan(root, index).await
}

async fn handle_path(
    path: &Path,
    collections: &Arc<tokio::sync::RwLock<Vec<Collection>>>,
    index: &EventIndex,
) -> Result<()> {
    if path.extension().and_then(|s| s.to_str()) != Some("ics") {
        return Ok(());
    }
    let snapshot = collections.read().await;
    let Some(collection) = snapshot.iter().find(|c| path.starts_with(&c.path)).cloned() else {
        return Ok(());
    };
    drop(snapshot);

    if !path.exists() {
        // File removed; we don't know its UID without parsing it.
        // By convention (pimsync writes one event per file with UID == filename stem),
        // we use the stem as the UID.
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            index
                .apply_change(IndexChange::Remove {
                    uid: stem.to_owned(),
                })
                .await;
        }
        return Ok(());
    }

    match std::fs::read_to_string(path) {
        Ok(text) => match parse_ics(&text, collection.id.clone()) {
            Ok(event) => {
                index.apply_change(IndexChange::upsert(event)).await;
            }
            Err(e) => warn!("parse {} failed: {e}", path.display()),
        },
        Err(e) => warn!("read {}: {e}", path.display()),
    }
    Ok(())
}
