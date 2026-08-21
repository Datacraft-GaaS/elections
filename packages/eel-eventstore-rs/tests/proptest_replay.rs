//! Property test: applying a shuffled sequence of commutative events yields the
//! same aggregate state as the ordered sequence.

use chrono::Utc;
use eel_eventstore_rs::aggregate::{Aggregate, AggregateSnapshot, StoredEvent};
use eel_eventstore_rs::error::EventStoreError;
use proptest::prelude::*;
use serde_json::{json, Value};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Sum aggregate (commutative under addition)
// ---------------------------------------------------------------------------

#[derive(Default)]
struct SumAggregate {
    id: Uuid,
    total: i64,
    version: i64,
}

impl Aggregate for SumAggregate {
    fn apply(&mut self, ev: &StoredEvent) -> Result<(), EventStoreError> {
        if let Some(v) = ev.payload.get("amount").and_then(Value::as_i64) {
            self.total += v;
        }
        self.version = ev.stream_version;
        Ok(())
    }

    fn snapshot(&self) -> AggregateSnapshot {
        AggregateSnapshot {
            stream_id: self.id,
            stream_version: self.version,
            state: json!({ "total": self.total }),
            taken_at: Utc::now(),
        }
    }

    fn stream_id(&self) -> Uuid {
        self.id
    }
}

fn make_event(stream_id: Uuid, version: i64, amount: i64) -> StoredEvent {
    StoredEvent {
        stream_id,
        stream_version: version,
        id: Uuid::new_v4(),
        event_type: "sum.amount_added.v1".into(),
        payload: json!({ "amount": amount }),
        prev_hash: None,
        hlc: format!("{}.0.1", 1_700_000_000_000i64 + version),
        recorded_at: Utc::now(),
    }
}

proptest! {
    #[test]
    fn shuffled_replay_matches_ordered(
        amounts in prop::collection::vec(-1000i64..=1000, 1..50),
        seed in 0u64..u64::MAX,
    ) {
        let stream_id = Uuid::new_v4();
        let ordered: Vec<StoredEvent> = amounts
            .iter()
            .enumerate()
            .map(|(i, &a)| make_event(stream_id, i as i64, a))
            .collect();

        // Apply in order.
        let mut agg_ordered = SumAggregate { id: stream_id, ..Default::default() };
        for ev in &ordered {
            agg_ordered.apply(ev).unwrap();
        }

        // Shuffle using a deterministic approach based on seed.
        let mut shuffled = ordered.clone();
        // Simple Fisher-Yates with lcg seeded by `seed`.
        let n = shuffled.len();
        let mut rng = seed;
        for i in (1..n).rev() {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let j = (rng >> 33) as usize % (i + 1);
            shuffled.swap(i, j);
        }

        // Apply shuffled (sum is commutative so totals must match).
        let mut agg_shuffled = SumAggregate { id: stream_id, ..Default::default() };
        for ev in &shuffled {
            agg_shuffled.apply(ev).unwrap();
        }

        prop_assert_eq!(agg_ordered.total, agg_shuffled.total);
    }
}

// ---------------------------------------------------------------------------
// Basic unit tests (no Docker)
// ---------------------------------------------------------------------------

#[test]
fn apply_empty_sequence() {
    let agg = SumAggregate::default();
    assert_eq!(agg.total, 0);
    let snap = agg.snapshot();
    assert_eq!(snap.state["total"], 0);
}

#[test]
fn apply_increments_total() {
    let stream_id = Uuid::new_v4();
    let mut agg = SumAggregate {
        id: stream_id,
        ..Default::default()
    };
    agg.apply(&make_event(stream_id, 0, 10)).unwrap();
    agg.apply(&make_event(stream_id, 1, 20)).unwrap();
    assert_eq!(agg.total, 30);
}
