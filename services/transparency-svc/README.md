# Transparency Service

Election Merkle transparency log with HSM-signed Signed Tree Heads, campaign finance disclosure pipeline, and reproducibility bundle generation.

## Purpose and modules

Operates the T1 election transparency log (module M14 reproducibility-bundle, M22 campaign-finance-transparency, M17 speech-integrity contributions only, plus P05 transparency log and P06 HSM signer). Publishes Signed Tree Heads at 1s/60s/5min cadences to five channels (IEBC website, GitHub, IPFS Kenya, Kenya Gazette print anchor, KBC Radio hash bulletin). Coordinates witness cosigning by IAB day-1 and external observers day-2. Manages campaign finance disclosure ingestion. Orchestrates HSM key ceremony coordination (break-glass).

Tier: T1 (transparency log and HSM signer), T2 (campaign finance).

Availability SLO: 99.99% (transparency log + HSM signer). STH publication latency p99 < 1s (1s cadence), < 60s (60s cadence).

RTO: 30 min. RPO: 5 min.

## Architecture overview

Core aggregates: TransparencyLog (append-only Merkle tree), SignedTreeHead (STH), CampaignDisclosure, ReproducibilityBundle. Backed by PostgreSQL (event store + snapshots) with Valkey caching for STH and bundle metadata. HSM integration via PKCS#11 for Ed25519 + ML-DSA hybrid signing. Kafka outbox relay for event publication. Receives Form 34 publications via RTS integration; publishes proof inclusion to transparency log on demand. Five-channel publisher handles asynchronous fanout to website, GitHub, IPFS Kenya, Gazette archives, and radio bulletin systems.

Upstream dependencies: rts-svc (form34.published, form34.submitted events), bvr-svc (register.frozen for result freeze window).

Downstream consumers: Witness systems (IAB + external observers), citizen-facing applications, audit tooling, reproducibility verification frameworks.

## Directory layout

```
services/transparency-svc/
  astro/                - Public transparency portal (Astro 5)
  crates/
    hsm-signer/         - HSM-backed signing service (PKCS#11)
    repro-bundle/       - Reproducibility bundle builder
  deploy/               - Kustomize base + overlays (primary, bomas, edge)
  django/               - Admin interface for log management
  docs/                 - Service documentation and runbooks
  loadtest/             - k6 scenarios
  migrations/           - Versioned PostgreSQL migrations
  proto/                - gRPC proto definitions
  src/                  - Main Rust binary (log server + coordinator)
  tests/                - Integration tests
  Cargo.toml, Cargo.lock
  data_dictionary.yaml
  TRACEABILITY.yaml
```

## Key APIs

REST (OpenAPI at packages/openapi-contracts/transparency/):
- GET /v1/tlog/sth: Retrieve latest Signed Tree Head
- GET /v1/tlog/sth/by-timestamp: STH by Unix timestamp (1s or 60s cadence)
- GET /v1/tlog/proof/{leaf_hash}: Inclusion proof for leaf hash
- GET /v1/tlog/consistency/{tree_size1}/{tree_size2}: Consistency proof between snapshots
- POST /v1/tlog/append (internal): Append leaf to log (RTS only)
- GET /v1/campaign-finance/disclosures: List campaign finance disclosure records
- GET /v1/campaign-finance/disclosures/{id}: Single disclosure
- GET /v1/repro-bundle/{election_id}: Download reproducibility bundle

gRPC (proto/transparency.proto):
- AppendLeaf(leaf_bytes) -> LeafIndex
- GetSTH(timestamp_ns) -> STH
- PublishSTH(sth) -> PublicationResult (witness coordination)
- IngestCampaignDisclosure(form_data) -> DisclosureId

## Key events emitted and consumed

Emitted:
- tlog.leaf.appended: Log leaf added to transparency log
- tlog.sth.published: STH signed and ready for publication
- tlog.witness.cosigned: IAB or external witness cosigned STH
- campaign.disclosure.ingested: Campaign finance form processed
- hsm.key.rotated: HSM key rotation completed
- tlog.sth.published.channel.{website,github,ipfs,gazette,radio}: Per-channel publication status

Consumed:
- rts.form34.published: Result form published; triggers transparency log append
- rts.form34.submitted: Form submitted to secondary queue
- bvr.register.frozen: Voter register freeze window for result acceptance

## Data model

Consult migrations/ for versioned PostgreSQL schema. data_dictionary.yaml defines:
- tlog_leaves: Log entries (hash, timestamp, station_id, evidence_hash)
- tlog_sths: Signed Tree Heads (tree_size, timestamp, signature, witness_sigs)
- campaign_disclosures: Campaign finance records (filer, amount, category, timestamp)
- repro_bundles: Bundle manifest (election_id, primary_hash, bundle_signature, timestamp)

HSM key material stored in OpenBao; signing operations audit-logged to PostgreSQL.

## Building and running

Build the main service:
```bash
cd /Users/nyimbiodero/src/pjs/eel/services/transparency-svc
cargo build --release
cargo build --release --bin hsm-signer  # HSM signer binary
```

Run locally with test transparency log:
```bash
cargo run --bin transparency-svc -- --config dev.toml --hms-key-file test-hsm-key.pem
```

Build reproducibility bundle generator:
```bash
cd crates/repro-bundle && cargo build --release
```

For web portal:
```bash
cd astro && pnpm install && pnpm build
pnpm preview  # Local preview
```

## Testing

Unit and property tests:
```bash
cargo test --lib --all
```

Integration tests (requires testcontainers, PostgreSQL, Kafka):
```bash
cargo nextest run --package transparency-svc
```

Contract tests (OpenAPI spec compliance):
```bash
cargo test --test '*_contract' -- --nocapture
```

Load testing (k6):
```bash
cd loadtest && k6 run sth-publication.js --vus 100 --duration 300s
```

Test tags: unit, integration, contract, load, chaos. Run via:
```bash
cargo nextest run --tag integration
```

## Deployment

Kustomize overlays at deploy/:
- deploy/base: Service deployment, PDB, HPA
- deploy/overlays/primary: Anniversary Towers (primary)
- deploy/overlays/bomas: Bomas mirror (hot standby)
- deploy/overlays/edge: Edge clusters (Nairobi, Mombasa, Kisumu, Eldoret, Garissa)

HPA targets CPU 70%, memory 80%; min replicas 3, max 10.

PDB: minAvailable 1 (always 2+ replicas running).

ApplicationSet at deploy/applicationset.yaml triggers Kustomize overlay build per environment.

PostgreSQL replication: primary async to bomas; bomas to edge read-replicas via streaming replication.

## Observability

Dashboards (Grafana):
- Transparency Log Metrics: leaf append latency, STH publication latency, queue depth, witness response time
- HSM Health: signing latency p50/p95/p99, key rotation status, HSM uptime
- Campaign Finance Ingestion: form processing rate, error rate, audit trail

Alerts (Prometheus):
- STH publication latency > 1s (1s cadence) / > 60s (60s cadence): critical
- HSM unavailable: critical (fallback to degraded mode)
- PostgreSQL replication lag > 5 min: warning
- Campaign finance queue depth > 10k: warning

Key metrics:
- tlog_leaf_append_duration_seconds
- tlog_sth_publication_duration_seconds
- tlog_sth_publication_delay_seconds (schedule vs actual)
- hsm_sign_duration_seconds
- campaign_disclosure_ingestion_rate
- witness_cosign_response_time_seconds

## Runbooks

See docs/runbooks/transparency-degradation.md for:
- HSM unavailable recovery
- PostgreSQL failover to bomas
- Network partition (primary to secondary)
- STH publication lag (queue backpressure)
- Witness cosigning timeout

## Degradation contract

Normal: Full log ingestion, HSM signing, 5-channel publication.

Degraded: If HSM unreachable, STH publication pauses; pending queue held in PostgreSQL; resumes on reconnect. Campaign finance read-only. Log reads from replica continue.

Read-only: Log read and proof endpoints serve from replica; no new appends; HSM offline; campaign finance frozen.

## Chaos scenarios

- hsm-unavailable: Kill HSM signer; verify queue holds; check resumption on restart
- network-partition-primary: Isolate primary from secondary; observe fallback
- tlog-replay-from-backup: Kill primary database; recover from bomas replica
- witness-timeout: Delay witness cosigning 30s+; verify queue backpressure
- sth-publication-lag: Saturate publication channels; verify batching and retry

## Statutes

Elections Act §39(1C) transmission audit trail and integrity.
Elections Act §83-84 declaration and certification with evidence.
Elections (Technology) Regulations 2017 audit trail requirements.

## Follow-ups

1. Integrate Ken HAL 3000 witness service (phase 2).
2. Add real-time witness cosigning status dashboard.
3. Implement tiered STH retention policy (1s every day, 60s every month, 5min archived).
4. Campaign finance witness configuration (phase 2).
5. Offline STH verification tool for radio bulletin recipients.
