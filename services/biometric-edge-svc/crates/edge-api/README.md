# edge-api

Kit-facing REST API for biometric edge devices. Handles attestation submission, configuration distribution, enrolment and identification event ingestion, offline queue sync, and diaspora station management.

## What it does

- Attestation endpoint: Accepts Android Integrity verdictToken or iOS device token; validates via platform service; returns kit registration status + SPIRE SVID
- Config distribution: Serves kit-specific configuration bundles (register extract, biometric params, certificate revocation list) with hash verification
- Event ingest: Accepts batch enrolment (fingerprint, iris, face, NID templates) and identification events (voter_id, station, 1:1 score); validates, writes to PG, publishes to Kafka
- Offline sync: Processes queued events from kit's 7-day local cache; merges with cloud state; returns delta for next poll period
- Kit registry: Maintains kit metadata (serial, hardware_id, Android version, app version, last sync) for fleet management
- Diaspora station API: Distributes station config; accepts Form34AD submissions; handles PouchCustody recording

## Public API surface

```rust
pub struct AttestationRequest {
    pub kit_serial: String,
    pub platform: Platform,
    pub token: Vec<u8>,
}

pub struct AttestationResponse {
    pub kit_status: KitStatus,
    pub spire_svid: String,
}

pub struct ConfigBundle {
    pub extract_version: String,
    pub extract_hash: String,
    pub enrolment_params: Map<String, Value>,
    pub identification_params: Map<String, Value>,
    pub cert_revocation_list: Vec<String>,
}

pub async fn attest_kit(req: AttestationRequest) -> Result<AttestationResponse>;
pub async fn get_config(kit_serial: &str) -> Result<ConfigBundle>;
pub async fn submit_enrolments(kit_serial: &str, events: Vec<EnrolmentEvent>) -> Result<()>;
pub async fn submit_identifications(kit_serial: &str, events: Vec<IdentificationEvent>) -> Result<()>;
pub async fn sync_offline_queue(kit_serial: &str, queued_events: Vec<Event>) -> Result<DeltaBundle>;
```

## Feature flags

None.

## Test commands

```bash
# Unit tests (attestation validation, config bundling, event parsing)
cargo test --lib --package eel-edge-api

# Integration tests (HTTP server, PG kit registry, Kafka event publication)
cargo test --test '*' --package eel-edge-api -- --test-threads=1

# Contract tests (kit payload schemas, API compliance)
cargo test --test contract --package eel-edge-api -- --nocapture
```

## Bench commands

```bash
# Attestation verification latency (100 tokens)
cargo bench --package eel-edge-api -- attest_kit

# Config bundle fetch (compressed extract + params)
cargo bench --package eel-edge-api -- get_config

# Batch event ingest (1000 enrolments)
cargo bench --package eel-edge-api -- submit_enrolments

# Offline queue sync (7-day backlog, delta computation)
cargo bench --package eel-edge-api -- sync_offline_queue
```

## Dependencies

- `axum` - HTTP server framework, routing
- `sqlx` - PostgreSQL kit registry, config cache, event writes
- `serde_json` - Event payload deserialization
- `tokio` - Async runtime
- `rdkafka` - Kafka event publication
- `uuid` - Event ID generation

## Related docs

- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - Kit sync flow diagram, config bundling, offline queue design
- [EDGE-CASES.md](../../docs/EDGE-CASES.md) (to be created) - Offline queue merge conflicts, duplicate event detection
