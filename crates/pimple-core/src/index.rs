//! In-memory event index with broadcast change notifications.

use std::collections::HashMap;
use std::sync::Arc;

use jiff::Timestamp;
use tokio::sync::{broadcast, RwLock};

use crate::error::Result;
use crate::event::{Event, EventInstance};
use crate::ical::expand::expand_in_range;
use crate::id::CollectionId;

/// One mutation to the index.
#[derive(Debug, Clone)]
pub enum IndexChange {
    /// Insert or replace the event with this UID.
    Upsert { uid: String, event: Box<Event> },
    /// Drop the event with this UID.
    Remove { uid: String },
    /// Replace the entire index. Used on root rescan.
    FullReload { events: Vec<Event> },
}

impl IndexChange {
    #[must_use]
    pub fn upsert(event: Event) -> Self {
        Self::Upsert {
            uid: event.uid.clone(),
            event: Box::new(event),
        }
    }
}

#[derive(Clone)]
pub struct EventIndex {
    inner: Arc<RwLock<HashMap<String, Event>>>,
    tx: broadcast::Sender<IndexChange>,
}

impl Default for EventIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl EventIndex {
    #[must_use]
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            tx,
        }
    }

    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<IndexChange> {
        self.tx.subscribe()
    }

    /// Apply a single change and notify subscribers.
    ///
    /// Lagging subscribers (slow UI) are tolerated — `broadcast::send` will not
    /// block. Receivers that miss messages will see `RecvError::Lagged` and can
    /// re-query the index.
    pub async fn apply_change(&self, change: IndexChange) {
        {
            let mut map = self.inner.write().await;
            match &change {
                IndexChange::Upsert { uid, event } => {
                    map.insert(uid.clone(), (**event).clone());
                }
                IndexChange::Remove { uid } => {
                    map.remove(uid);
                }
                IndexChange::FullReload { events } => {
                    map.clear();
                    for event in events {
                        map.insert(event.uid.clone(), event.clone());
                    }
                }
            }
        }
        let _ = self.tx.send(change);
    }

    /// Return all `EventInstance`s in `[range_start, range_end)` whose
    /// collection is in `visible`.
    ///
    /// # Errors
    /// Propagates `CoreError` from RRULE expansion.
    pub async fn events_in_range(
        &self,
        range_start: Timestamp,
        range_end: Timestamp,
        visible: &[CollectionId],
    ) -> Result<Vec<EventInstance>> {
        let map = self.inner.read().await;
        let mut out = Vec::new();
        for event in map.values() {
            if !visible.contains(&event.collection_id) {
                continue;
            }
            out.extend(expand_in_range(event, range_start, range_end)?);
        }
        out.sort_by_key(|i| i.start.to_timestamp());
        Ok(out)
    }
}
