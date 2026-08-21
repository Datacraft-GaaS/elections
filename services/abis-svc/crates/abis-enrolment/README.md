# abis-enrolment

Enrolment lifecycle and 1:N deduplication pipeline. Processes biometric template submissions from the voter register, runs duplicate detection, and emits acceptance or rejection verdicts.

## What it does

- Template ingestion: Listens for `bvr.template.submitted.v1` events from voter enrolment kit; extracts template vector via vendor gRPC call
- Quality checks: Validates template dimensions, feature count, and freshness; rejects low-quality scans
- Dedup trigger: Routes cleaned template to `abis-matcher` for 1:N gallery search
- Verdict emission: Publishes `abis.enrolment.accepted.v1` or `abis.enrolment.rejected.v1` back to BVR
- Enrolment timeline: Tracks voter through states (submitted, processing, deduplicated, adjudicated, accepted/rejected) in `eel_enrolment_status` table
- Batch commit: Every 100 accepted enrolments, writes STH (Signed Tree Hash) entry and tlog anchor; emits batch_committed event

## Public API surface

```rust
pub struct EnrolmentRequest {
    pub voter_id: String,
    pub template: Vec<u8>,
    pub template_type: BiometricType,
    pub kit_serial: String,
}

pub struct EnrolmentResult {
    pub voter_id: String,
    pub status: EnrolmentStatus,
    pub duplicate_of: Option<String>,
    pub adjudication_case_id: Option<String>,
}

pub enum EnrolmentStatus { Submitted, Processing, Accepted, Rejected, Adjudicated }

pub async fn ingest_template(req: EnrolmentRequest) -> Result<EnrolmentResult>;
pub async fn check_quality(template: &[u8]) -> Result<QualityScore>;
pub async fn emit_acceptance(voter_id: &str) -> Result<()>;
pub async fn commit_batch() -> Result<BatchCommitReceipt>;
```

## Feature flags

None.

## Test commands

```bash
# Unit tests (quality score calculation, state machine transitions)
cargo test --lib --package eel-abis-enrolment

# Integration tests (Kafka event consumption, PG timeline writes, batch commit)
cargo test --test '*' --package eel-abis-enrolment -- --test-threads=1
```

## Bench commands

```bash
# Template quality check latency (1000 templates)
cargo bench --package eel-abis-enrolment -- quality_check

# Batch commit time (100 acceptances, STH entry, tlog anchor)
cargo bench --package eel-abis-enrolment -- commit_batch
```

## Dependencies

- `tokio` - Async Kafka consumer, channel-based worker pool
- `rdkafka` - Kafka consumer for BVR template events
- `sqlx` - Timeline state machine, batch commit queries
- `serde` - Event deserialization
- `uuid` - Timeline event ID generation

## Related docs

- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - Template ingestion flow, batch commit diagram, tlog anchoring
- [EVENTS.md](../../docs/EVENTS.md) - Template submission and verdict event schemas, ordering guarantees
- [EDGE-CASES.md](../../docs/EDGE-CASES.md) - Duplicate batch submissions, tlog anchor retry logic
