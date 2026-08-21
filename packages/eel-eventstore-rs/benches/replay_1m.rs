//! 1M-event in-memory replay benchmark.
//!
//! Against a real Postgres instance this measures end-to-end replay latency.
//! Without Docker the benchmark runs against an in-memory counter to validate
//! that the hot path (apply loop) itself is fast.

use chrono::Utc;
use criterion::{criterion_group, criterion_main, Criterion};
use eel_eventstore_rs::aggregate::{Aggregate, AggregateSnapshot, StoredEvent};
use eel_eventstore_rs::error::EventStoreError;
use serde_json::{json, Value};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Minimal aggregate for benchmarking
// ---------------------------------------------------------------------------

struct CounterAggregate {
    id: Uuid,
    count: u64,
    version: i64,
}

impl Default for CounterAggregate {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            count: 0,
            version: -1,
        }
    }
}

impl Aggregate for CounterAggregate {
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError> {
        if let Some(delta) = event.payload.get("delta").and_then(Value::as_u64) {
            self.count += delta;
        }
        self.version = event.stream_version;
        Ok(())
    }

    fn snapshot(&self) -> AggregateSnapshot {
        AggregateSnapshot {
            stream_id: self.id,
            stream_version: self.version,
            state: json!({ "count": self.count }),
            taken_at: Utc::now(),
        }
    }

    fn stream_id(&self) -> Uuid {
        self.id
    }
}

// ---------------------------------------------------------------------------
// Benchmark
// ---------------------------------------------------------------------------

fn build_events(n: usize) -> Vec<StoredEvent> {
    let stream_id = Uuid::new_v4();
    (0..n)
        .map(|i| StoredEvent {
            stream_id,
            stream_version: i as i64,
            id: Uuid::new_v4(),
            event_type: "counter.incremented.v1".into(),
            payload: json!({ "delta": 1u64 }),
            prev_hash: None,
            hlc: format!("{}.0.1", 1_700_000_000_000u64 + i as u64),
            recorded_at: Utc::now(),
        })
        .collect()
}

fn bench_replay_1m(c: &mut Criterion) {
    let events = build_events(1_000_000);

    c.bench_function("replay_1m_in_memory", |b| {
        b.iter(|| {
            let mut agg = CounterAggregate::default();
            for ev in &events {
                agg.apply(ev).unwrap();
            }
            assert_eq!(agg.count, 1_000_000);
        });
    });
}

criterion_group!(benches, bench_replay_1m);
criterion_main!(benches);
