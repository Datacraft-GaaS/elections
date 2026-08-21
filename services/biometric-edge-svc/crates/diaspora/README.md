# diaspora

Diaspora embassy voting station protocol. Implements M24 diaspora voting workflows: embassy station management, DiasporaRoll event batching, Form34AD closure submission, and PouchCustody for secure transport of physical ballots and materials.

## What it does

- Station management: Registers embassy voting stations with timezone, staff roles, and observer list
- DiasporaRoll: Event batch structure containing voter roster, identification events, and turnout aggregate
- Form34AD submission: Collects election closure results (total votes cast, spoiled ballots, seal checksums) with dual signature (staff + observer)
- PouchCustody: Audit trail for physical ballot pouch seal and handover (timestamp, GPS, signatories, custody chain)
- Event emission: Publishes `biometric-edge.diaspora.roll.submitted.v1` and `biometric-edge.diaspora.form34ad.submitted.v1` to Kafka
- Offline queue: Queues rolls if backend unreachable; retries on next sync with exponential backoff

## Public API surface

```rust
pub struct EmbassyStation {
    pub station_id: String,
    pub country: String,
    pub timezone: String,
    pub staff_roles: Vec<StaffRole>,
}

pub struct DiasporaRoll {
    pub station_id: String,
    pub roll_date: Date,
    pub total_identified: usize,
    pub identifications: Vec<IdentificationEvent>,
}

pub struct Form34AD {
    pub station_id: String,
    pub poll_open_time: DateTime,
    pub poll_close_time: DateTime,
    pub total_votes_cast: usize,
    pub spoiled_ballots: usize,
    pub staff_signature: String,
    pub observer_signature: String,
    pub seal_checksums: Vec<String>,
}

pub struct PouchCustody {
    pub pouch_id: String,
    pub seal_hash: String,
    pub created_at: DateTime,
    pub custodian: String,
    pub gps_location: (f64, f64),
    pub signature: String,
}

pub async fn register_station(station: EmbassyStation) -> Result<()>;
pub async fn submit_roll(roll: DiasporaRoll) -> Result<()>;
pub async fn submit_form34ad(form: Form34AD) -> Result<()>;
pub async fn record_pouch_custody(custody: PouchCustody) -> Result<()>;
pub async fn get_station_config(station_id: &str) -> Result<EmbassyStation>;
```

## Feature flags

None.

## Test commands

```bash
# Unit tests (Form34AD signature validation, pouch custody chain, roll structure)
cargo test --lib --package eel-diaspora

# Integration tests (PostgreSQL diaspora tables, Kafka event emission)
cargo test --test '*' --package eel-diaspora -- --test-threads=1

# Property tests (custody chain invariants, dual-signature validation)
cargo test --package eel-diaspora -- --nocapture
```

## Bench commands

```bash
# Roll submission (batch 1000 identification events)
cargo bench --package eel-diaspora -- submit_roll

# Form34AD validation (dual signature verification)
cargo bench --package eel-diaspora -- form34ad_verify

# Pouch custody recording (GPS logging, signature)
cargo bench --package eel-diaspora -- record_custody
```

## Dependencies

- `sqlx` - PostgreSQL diaspora table writes (station config, rolls, forms)
- `serde` - Form34AD and DiasporaRoll serialization
- `ed25519-dalek` - Dual-signature verification for Form34AD
- `sha2` - Hash seal verification
- `tokio` - Async API handlers, event publication

## Related docs

- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - Diaspora flow diagram, Form34AD dual-control design
- [OPERATIONS.md](../../docs/OPERATIONS.md) - Embassy station onboarding, Form34AD manual submission
