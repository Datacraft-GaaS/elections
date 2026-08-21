# Results Transmission System

Election results processing from 46,229 polling stations with formal tally verification, digital declarations, and ballot serial tracking.

## Purpose and modules

Results Transmission System (RTS). Processes and publishes election results from all polling stations. Module M08 rts-device scans and signs Form 34A/B/C at kit-side. Module M09 rts-ro (RO workstation) desktop app handles ICR extraction and result disputes. Module M10 rts-backend stores, verifies, tallies, and publishes results to transparency log. Module M51 issues digital declarations and certificates of election. Module M63 tracks ballot serial number chain of custody.

Tier: T1 (election-critical).

Availability SLO: 99.99%. Submit latency p95 < 500ms, p99 < 1s at 60k concurrent stations. Ingest latency p99 < 2s.

RTO: 30 min. RPO: 5 min.

## Architecture overview

Core aggregates: Form34A/B/C (result forms), PublicationSnapshot (tally state), MerkleRoot (inclusion proof), Declaration (HSM-signed), BallotSerial (issue and reconciliation). Backed by PostgreSQL (event store + snapshots, with JSONB for flexible form schema). Receives form submissions from M08 kits and M09 RO workstations via REST/SMS fallback. TLA+-verified tally logic handles run-off, tie, referendum threshold (50%+1 national, majority per county) scenarios. HSM signing for declarations. Kafka outbox relay publishes to transparency-svc.

Upstream dependencies: bvr-svc (register.frozen event gates result acceptance), transparency-svc (sth.published for inclusion proofs).

Downstream consumers: transparency-svc (results publication), citizen apps, election observers, audit systems, declaration authorities.

## Directory layout

```
services/rts-svc/
  apps/                 - Tauri desktop app (rts-ro M09 workstation)
  crates/
    ballot-serial/      - M63 ballot serial chain tracking
    declaration/        - M51 declaration and certificate
    rts-api/            - Public REST + gRPC API
    rts-device-api/     - M08 kit-facing API (form submission, SMS)
    rts-domain/         - Core aggregates, TLA+-verified tally
    rts-integration-tests/ - Integration test harness
    rts-projections/    - Read-model projections (running tally)
    rts-publisher/      - Transparency log publisher, Kafka relay
    rts-ro-api/         - M09 RO workstation API (ICR, disputes)
    rts-tally/          - TLA+-verified tally aggregation engine
  deploy/               - Kustomize base + per-site overlays
  django/               - Admin interface (bulk ops, adjudication)
  docs/                 - Service documentation
  loadtest/             - k6 + Gatling scenarios (60k concurrent)
  migrations/           - Versioned PostgreSQL migrations
  py/                   - Python ML (ICR handwriting recognition)
  proto/                - gRPC proto definitions
  src/                  - Main Rust binary (result server)
  tests/                - Integration + e2e test harnesses
  Cargo.toml, Cargo.lock
  data_dictionary.yaml
  TRACEABILITY.yaml
```

## Key APIs

REST (OpenAPI at packages/openapi-contracts/rts/):
- POST /v1/rts/stations/{station_id}/form34: Submit Form 34A/B/C from kit or workstation
- GET /v1/rts/stations/{station_id}/form34: Retrieve latest form
- GET /v1/rts/tally/{constituency_id}: Running tally for constituency
- GET /v1/rts/tally/summary: National tally summary
- GET /v1/rts/declaration/{election_id}: Issued declaration and certificate
- POST /v1/rts/ballot-serials/assign: Assign serial numbers to ballot batch
- GET /v1/rts/ballot-serials/status: Ballot reconciliation status
- POST /v1/rts/disputes/raise: RO raises result dispute (M09)

gRPC (proto/rts.proto):
- SubmitForm34(form_bytes, sig) -> FormReceipt
- GetTally(constituency_id) -> TallySnapshot
- SignDeclaration(tally) -> Declaration
- AssignBallotSerials(batch) -> AssignmentReceipt

## Key events emitted and consumed

Emitted:
- rts.form34.submitted: Form 34 received at kit or workstation
- rts.form34.published: Form validated, stored, included in tally
- rts.tally.updated: Tally recalculated (station result change or dispute resolution)
- rts.declaration.signed: Declaration issued with HSM signature
- rts.ballot.serial.assigned: Ballot batch assigned serial range
- rts.ballot.serial.reconciled: Spoilt ballots accounted for

Consumed:
- bvr.register.frozen: Voter register freeze window; gates result acceptance
- transparency.sth.published: STH published; used for inclusion proofs

## Data model

Consult migrations/ for versioned PostgreSQL schema. data_dictionary.yaml defines:
- form34_submissions: Station ID, form type (A/B/C), vote counts, timestamp, signature
- tally_snapshots: Constituency/national tally, tree_size, merkle_root, timestamp
- declarations: Election ID, tally snapshot hash, HSM signature, witness sigs
- ballot_serials: Batch ID, serial range, issued count, spoilt count, reconciliation_hash

TLA+ models verify tally correctness, referendum threshold, run-off, and tie scenarios.

## Building and running

Build the main service:
```bash
cd /Users/nyimbiodero/src/pjs/eel/services/rts-svc
cargo build --release
```

Build the RO desktop app (M09):
```bash
cd apps/rts-ro && npm install && npm run build
```

Run the server locally:
```bash
cargo run --bin rts-svc -- --config dev.toml --db-url postgres://localhost/rts_dev
```

Build ML component (ICR):
```bash
cd py && pip install -e . && python -m icr.train
```

## Testing

Unit and property tests:
```bash
cargo test --lib --all
```

TLA+ formal verification (tally logic):
```bash
cd docs && apalache check tally.tla
```

Integration tests (testcontainers):
```bash
cargo nextest run
```

Contract tests (OpenAPI compliance):
```bash
cargo test --test '*_contract'
```

Load testing (60k concurrent):
```bash
cd loadtest && gatling -s RtsLoadSimulation
k6 run form34-submit.js --vus 60000 --duration 600s
```

## Deployment

Kustomize overlays at deploy/:
- deploy/base: Service deployment, PDB (minAvailable 2), HPA
- deploy/overlays/primary: Anniversary Towers (primary, 3+ replicas)
- deploy/overlays/bomas: Bomas mirror (read-only, 2+ replicas)
- deploy/overlays/nairobi, mombasa, kisumu, eldoret, garissa: Edge read replicas

HPA targets CPU 75%, memory 80%; min 3, max 30 replicas (primary); scales via request rate.

PDB: minAvailable 2 (always 3+ replicas safe during disruption).

ApplicationSet at deploy/applicationset.yaml triggers per-environment Kustomize build.

PostgreSQL replication: primary async to bomas (< 5s lag); bomas to edge read-replicas.

## Observability

Dashboards (Grafana):
- RTS Submit: form submission rate, latency p50/p95/p99, error rate
- RTS Tally: running tally, dispute cases, tie/run-off scenarios
- Declaration Pipeline: signing latency, witness cosigning status
- Ballot Serial: assignment rate, reconciliation status, spoilt count

Alerts (Prometheus):
- Form submit latency p99 > 1s: warning
- Tally consistency violation: critical
- Declaration signing latency > 500ms: warning
- Ballot serial reconciliation failed: critical
- PostgreSQL replication lag > 5 min: warning

Key metrics:
- rts_form34_submit_duration_seconds
- rts_form34_submission_rate
- rts_form34_submit_errors
- rts_tally_consistency_checks_passed
- rts_declaration_sign_duration_seconds
- rts_ballot_serial_assignment_rate
- rts_ballot_serial_reconciliation_status

## Runbooks

See docs/runbooks/rts-degradation.md for:
- Form submission queue backpressure recovery
- Tally consistency violation investigation
- HSM unavailable during declaration (fallback to ECC manual sign)
- PostgreSQL primary failover to bomas
- Edge site result sync lag

## Degradation contract

Normal: Full ingestion, tally verification, declaration signing, transparency log publication.

Degraded: Accept submissions to queue; halt tally publication; queue pending in PostgreSQL; alert ECC. No new declarations. Form reads from replica; declaration reads stale (latest published).

Read-only: Serve last published tally and declaration; halt ingestion and signing.

## Chaos scenarios

- kafka-partition: Partition Kafka; verify queue holds forms; resume on reconnect
- rts-primary-down: Kill primary RTS server; verify failover to read-only, form queue persists
- hsm-unavailable-during-declaration: HSM offline; declaration delayed; ECC manual fallback
- tlog-publish-lag: Saturate transparency log publisher; RTS tally blocks (backpressure)
- ballot-serial-mismatch: Mock spoilt count mismatch; verify reconciliation escalation

## Statutes

Elections Act §39(1C) transmission audit trail and integrity verification.
Elections Act §§83-84 declaration and certification of results.
Elections Act §80 recount and scrutiny procedures.
Elections Act §27 Form 34 and Form 35 tally procedures.

## Follow-ups

1. Real-time M08 kit telemetry dashboard (phase 2).
2. Automated dispute resolution with ML confidence scoring (phase 2).
3. SMS form fallback route (legacy kit support).
4. Ballot serial forensics (reconciliation audit tool).
5. Observer cosigning of declarations (phase 2).
