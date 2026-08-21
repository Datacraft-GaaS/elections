use eel_hlc_rs::Hlc;
use uuid::Uuid;

use crate::{
    aggregate::{Aggregate, AggregateSnapshot, StoredEvent},
    error::EventStoreError,
    store::EventStore,
};

/// Iterator-style replay over a slice of stored events.
pub struct EventReplay<'a> {
    events: &'a [StoredEvent],
    pos: usize,
    after: Option<Hlc>,
}

impl<'a> EventReplay<'a> {
    pub fn new(events: &'a [StoredEvent], after: Option<Hlc>) -> Self {
        Self { events, pos: 0, after }
    }
}

impl<'a> Iterator for EventReplay<'a> {
    type Item = &'a StoredEvent;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos >= self.events.len() {
                return None;
            }
            let ev = &self.events[self.pos];
            self.pos += 1;
            if let Some(ref cutoff) = self.after {
                if let Ok(ev_hlc) = ev.hlc.parse::<Hlc>() {
                    if ev_hlc <= *cutoff {
                        continue;
                    }
                }
            }
            return Some(ev);
        }
    }
}

/// Snapshot-then-tail helper.
///
/// Restores an aggregate from a snapshot (if one exists) and then applies any
/// trailing events from the store.
pub struct SnapshotTailReplay<'store> {
    store: &'store EventStore,
}

impl<'store> SnapshotTailReplay<'store> {
    pub fn new(store: &'store EventStore) -> Self {
        Self { store }
    }

    /// Rebuild an aggregate for `stream_id`.
    ///
    /// If a snapshot is found, `restore_fn` converts it to `A`; events after
    /// that snapshot version are then applied.  Without a snapshot, all events
    /// from the beginning are applied to `A::default()`.
    pub async fn rebuild<A, F>(
        &self,
        stream_id: Uuid,
        restore_fn: F,
    ) -> Result<A, EventStoreError>
    where
        A: Aggregate + Default,
        F: FnOnce(AggregateSnapshot) -> A,
    {
        let snap = self.store.load_snapshot(stream_id).await?;

        match snap {
            None => {
                let events = self.store.load_events(stream_id, None).await?;
                let mut agg = A::default();
                for ev in &events {
                    agg.apply(ev)?;
                }
                Ok(agg)
            }
            Some(s) => {
                let after_version = s.stream_version;
                let mut agg = restore_fn(s);
                // load events after the snapshot version
                let rows = self.store.load_events(stream_id, None).await?;
                for ev in rows.iter().filter(|e| e.stream_version > after_version) {
                    agg.apply(ev)?;
                }
                Ok(agg)
            }
        }
    }
}
