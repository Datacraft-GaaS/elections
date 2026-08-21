# abis-matcher

Core matching engine. Routes match requests across shard tiers (hot, warm, cold), coordinates vendor SDK calls, and applies threshold scoring to produce match verdicts.

## What it does

- Async worker: Pulls match requests from Kafka `eel-abis-match` topic via consumer group
- Shard routing: Fans out 1:N query across Tier 1, 2, and 3 HNSW gallery shards; merges candidate lists by score
- Vendor coordination: Calls `abis-vendor` gRPC service (port 8090) to score probe against candidates; implements circuit breaker for vendor failures
- Threshold decision: Applies configurable lower/upper thresholds; routes scores in [lower, upper] to adjudication queue
- Result write: Stores match result row with score, decision, vendor call time, and attestation hash
- Event emission: Publishes `abis.match.completed.v1` or `abis.adjudication.case.created.v1` to Kafka via outbox relay

## Public API surface

```rust
pub struct MatchRequest {
    pub probe_template: Vec<u8>,
    pub gallery_shard: Option<u8>,
    pub vendor: String,
}

pub struct MatchResult {
    pub match_id: String,
    pub score: f32,
    pub decision: MatchDecision,
    pub top_candidates: Vec<Candidate>,
    pub vendor_latency_ms: u32,
}

pub enum MatchDecision { Match, NoMatch, Adjudication }

pub async fn submit_match(req: MatchRequest) -> Result<String>;
pub async fn run_matcher_worker();
pub async fn search_shard(shard_tier: u8, probe: &[u8]) -> Result<Vec<Candidate>>;
```

## Feature flags

None.

## Test commands

```bash
# Unit tests (shard routing, threshold logic, candidate merge)
cargo test --lib --package eel-abis-matcher

# Integration tests (Kafka consumer, vendor mock, shard simulation)
cargo test --test '*' --package eel-abis-matcher -- --test-threads=1

# Property tests (threshold boundary conditions, score distribution)
cargo test --package eel-abis-matcher -- --nocapture
```

## Bench commands

```bash
# 1:N search latency across Tier 1, 2, 3 at scale
cargo bench --package eel-abis-matcher -- search_latency

# Vendor gRPC call overhead (100 calls, measure round-trip time)
cargo bench --package eel-abis-matcher -- vendor_call

# Candidate merge time (1000 candidates across 3 tiers)
cargo bench --package eel-abis-matcher -- merge_candidates
```

## Dependencies

- `tokio` - Async runtime, Kafka consumer group coordination
- `rdkafka` - Kafka topic subscription and consumer group offset management
- `tonic` - gRPC client for vendor service calls
- `sqlx` - Result row write, gallery shard metadata queries
- `proptest` - Property-based testing of score distributions

## Related docs

- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - Shard tier diagram, vendor routing flow, circuit breaker config
- [BENCHMARKS.md](../../docs/BENCHMARKS.md) - HNSW ef tuning per tier, match latency targets by gallery size
- [OPERATIONS.md](../../docs/OPERATIONS.md) - Shard tier recovery, vendor circuit breaker manual reset
