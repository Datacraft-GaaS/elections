# abis-adjudication

Adjudication workflow, FSM, and reviewer audit chain for borderline biometric matches in the ABIS.

When a biometric match score falls between lower and upper thresholds (e.g., 0.75-0.90), the system creates an adjudication case that requires human review. This crate implements the case state machine, priority queue, SLA tracking, and immutable audit trail of all reviewer actions.

## What it does

- Case aggregate: `AdjudicationCase` entity with FSM states (Created, Assigned, InReview, Approved/Rejected)
- Priority queue: Orders cases by urgency (SLA deadline, submission time)
- SLA tracking: Monitors first-response deadline (4h) and full-resolution deadline (24h)
- Audit trail: Records every action (create, assign, review, decide) with timestamp, actor, reason
- Dual-control: Enforces via SpiceDB that submitting officer cannot approve own case
- Case recovery: Idempotent upsert and re-entrancy safe under concurrent reviewer actions

## Public API surface

```rust
pub struct AdjudicationCase {
    pub id: String,
    pub match_id: String,
    pub status: CaseStatus,
    pub probe_template_id: String,
    pub candidate_template_id: String,
    pub score: f32,
    pub created_at: DateTime,
    pub assigned_to: Option<String>,
    pub sla_deadline_at: DateTime,
    pub decision: Option<CaseDecision>,
    pub decision_reason: Option<String>,
}

pub enum CaseStatus { Created, Assigned, InReview, Approved, Rejected }
pub enum CaseDecision { Match, NoMatch, Inconclusive }

pub struct AdjudicationRepo {
    pub async fn create(case: &AdjudicationCase) -> Result<()>;
    pub async fn get_by_id(id: &str) -> Result<AdjudicationCase>;
    pub async fn list_pending(limit: usize) -> Result<Vec<AdjudicationCase>>;
    pub async fn assign(case_id: &str, reviewer_id: &str) -> Result<()>;
    pub async fn decide(case_id: &str, decision: CaseDecision, reason: &str) -> Result<()>;
    pub async fn check_sla_breaches() -> Result<Vec<String>>;
}

pub async fn emit_audit_event(case_id: &str, action: &str, actor_id: &str) -> Result<()>;
```

## Feature flags

- `test-utils` - Enable `Ed25519TestSigner` for integration tests and external test crates

## Test commands

```bash
# Unit tests (in-memory FSM, audit trail mutation rules)
cargo test --lib --package eel-abis-adjudication

# Integration tests (PG schema, concurrent reviewer actions, SLA timer)
cargo test --test '*' --package eel-abis-adjudication -- --test-threads=1

# Property tests (FSM invariants, state transitions)
cargo test --package eel-abis-adjudication -- --nocapture
```

## Bench commands

```bash
# Case creation + audit append latency
cargo bench --package eel-abis-adjudication

# SLA breach scan (1k cases, measure query time)
cargo bench --package eel-abis-adjudication -- sla_scan
```

## Dependencies

- `sqlx` - PostgreSQL query execution, migrations, prepared statements
- `tokio` - Async runtime, spawn_blocking for SLA scan
- `serde` - Case/decision serialization for Kafka events
- `blake3` - Hash audit trail entries for immutability proof
- `ed25519-dalek` - Sign case decisions for non-repudiation
- `eel-crypto-rs` - HSM signer integration for countersignature

## Related docs

- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - Case flow diagram, dual-control enforcement via SpiceDB
- [OPERATIONS.md](../../docs/OPERATIONS.md) - Case recovery after reviewer crash, SLA escalation workflow
- [EDGE-CASES.md](../../docs/EDGE-CASES.md) - Concurrent assignment race conditions, duplicate case prevention
