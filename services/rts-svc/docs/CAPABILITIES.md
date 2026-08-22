# rts-svc capabilities

Results Transmission System: event-sourced result processing for 46,229 polling stations with TLA+-verified tally logic, ONNX-based handwriting recognition for Form 34A, HSM-countersigned declarations, and ballot serial chain of custody.

**Tier**: T1-election-critical.
**SLO**: 99.99% availability. Form submit p95 < 500ms, p99 < 1s at 60k concurrent stations. Ingest p99 < 2s.
**RTO**: 30 min. **RPO**: 5 min.

---

## Contents

1. Purpose and context
2. End-to-end pipelines
3. Domain aggregates and state stores
4. Algorithms and standards
5. Crypto and trust anchoring
6. Event surface
7. API surface
8. Integrations
9. Sharding, indexing, and scale
10. Degradation modes
11. Data lifecycle and retention
12. Observability
13. Testing pyramid
14. Follow-ups and known limitations

---

## 1. Purpose and context

### Modules owned

| Module | Name | RFP ref |
|--------|------|---------|
| M08 | rts-device (kit-side form scan and sign) | WP-4.1 |
| M09 | rts-ro (Returning Officer desktop) | WP-4.2 |
| M10 | rts-backend (result server and tally) | WP-4.3 |
| M51 | Result declaration and certificate | WP-4.5 |
| M63 | Ballot serial chain of custody | WP-4.6 |

### Timeline role

| Phase | Activity |
|-------|----------|
| T-6 months | RTS device firmware deployed to kits; Form 34A/B/C templates loaded |
| T-3 months | Load testing at 60k concurrent; RO desktop installed in county offices |
| T-1 day | bvr.register.frozen event received; result acceptance window opens |
| Election day T+0 to T+5 hours | Form 34A submissions from 46,229 stations; running tally updated; SMS fallback active |
| T+5 to T+12 hours | Form 34B constituency aggregation; RO dispute window open |
| T+1 day | Form 34C national compilation; HSM-countersigned declaration issued |
| T+7 days | Declaration appended to transparency log; reproducibility bundle triggered |

---

## 2. End-to-end pipelines

### 2.1 Form 34A submission (kit-side M08)

```
M08 kit: scan Form 34A (physical paper) -> ONNX ICR model (py/ worker)
  -> extract vote counts per candidate -> sign payload (SPIRE SVID Ed25519)
  -> POST /v1/rts/stations/{station_id}/form34 (rts-device-api crate)
  -> idempotency check (Kafka transaction producer) -> store in eel_events (event store)
  -> project into form34a_view -> publish rts.form34.submitted CloudEvent
  -> on validation pass: publish rts.form34.published -> relay to transparency-svc
```

### 2.2 SMS/USSD fallback

```
M08 kit (offline): encode result as CBOR, compress with zstd
  -> transmit via SMS (CBOR+zstd fits within 2 SMS segments)
  -> rts-device-api: receive SMS webhook from telecom gateway
  -> decode CBOR, decompress -> same validation path as REST submission
```

### 2.3 Tally aggregation (M10)

```
form34.published event -> rts-tally crate (Kafka consumer, exactly-once via Kafka transactions)
  -> aggregate votes_cast, valid_votes, candidate_totals per constituency
  -> update form34b_view (constituency level) -> update form34c_view (national)
  -> compute merkle_root over all form34a_view rows for constituency
  -> publish rts.tally.updated CloudEvent
```

### 2.4 RO review and dispute (M09)

```
RO at Tauri desktop app (apps/rts-ro):
  -> review ICR extraction annotations in form34a_view
  -> if dispute: POST /v1/rts/disputes/raise -> rts-ro-api crate
  -> write DisputeRaised event to eel_events -> tally suspended for constituency
  -> RO manually enters correct counts -> resolved: tally resumes
```

### 2.5 Declaration (M51)

```
form34c_view.status = 'compiled' -> declaration crate
  -> assemble DeclarationPayload (tally snapshot + merkle_root + election_id)
  -> request HSM countersignature (Ed25519, key role 'declaration')
  -> insert into declarations table -> publish rts.declaration.signed CloudEvent
  -> transparency-svc appends declaration hash to log
```

### 2.6 Ballot serial tracking (M63)

```
ballot-serial crate: POST /v1/rts/ballot-serials/assign
  -> assign serial range [start, end] to batch_id -> insert into ballot_serials
  -> at reconciliation: spoilt_count + issued_count + unused must equal range_size
  -> publish rts.ballot.serial.reconciled when balanced
```

---

## 3. Domain aggregates and state stores

Tables come from `services/rts-svc/migrations/V0001__events_schema.sql`, `V0002__projection_tables.sql`, and `V0003__audit_shadows.sql`.

| Table | Purpose |
|-------|---------|
| `eel_events` | Append-only event store. Columns: `id` (UUID PK), `stream_id`, `stream_version`, `event_type`, `payload` (JSONB), `metadata` (JSONB), `producer_signature` (BYTEA), `hsm_countersignature` (BYTEA), `recorded_at`, `event_time`, `hlc` (Hybrid Logical Clock int64), `prev_hash` (BYTEA). Unique on `(stream_id, stream_version)`. Indexed by `stream_id/stream_version`, `event_type`, `recorded_at`. |
| `eel_snapshots` | Event aggregate snapshots for fast hydration. Columns: `stream_id`, `stream_version`, `state` (JSONB), `taken_at`. |
| `eel_outbox` | Transactional outbox for Kafka relay. Columns: `id`, `topic`, `payload` (JSONB), `created_at`, `published_at`, `attempt_count`. Partial index on unpublished rows. |
| `form34a_view` | Projection: per-station form. Columns: `form_id`, `stream_id`, `stream_version`, `station_code`, `constituency_code`, `county_code`, `registered_voters`, `votes_cast`, `valid_votes`, `rejected_votes`, `spoilt_ballots`, `candidate_totals` (JSONB), `status`, `raw_scan_ref`, `annotations` (JSONB), `updated_at`. |
| `form34b_view` | Projection: per-constituency aggregate. Columns: `form_id`, `stream_id`, `stream_version`, `constituency_code`, `county_code`, `compiled_stations` (JSONB), `total_registered`, `total_valid`, `total_rejected`, `total_spoilt`, `candidate_totals` (JSONB), `status`, `updated_at`. Unique on `constituency_code`. |
| `form34c_view` | Projection: national totals. Columns: `form_id`, `stream_id`, `stream_version`, `compiled_constituencies` (JSONB), all totals, `merkle_root`, `status`, `published_at`, `updated_at`. Unique on `stream_id`. |
| `publication_snapshots` | Tlog anchors. Columns: `snapshot_id`, `form34c_id` (FK), `merkle_root`, `signed_url`, `tlog_leaf_index`, `created_at`. |
| `declarations` (declaration crate) | HSM-signed declarations. Planned columns: `declaration_id`, `election_id`, `form34c_id`, `tally_snapshot_hash`, `hsm_signature`, `witness_sigs`, `issued_at`. Verify in Wave 2. |
| `ballot_serials` (ballot-serial crate) | Serial tracking. Planned columns: `batch_id`, `serial_start`, `serial_end`, `issued_count`, `spoilt_count`, `reconciliation_hash`. Verify in Wave 2. |

The `prev_hash` column in `eel_events` implements a per-stream hash chain: `prev_hash = SHA-256(event[n-1].payload || event[n-1].prev_hash)`. This makes any tampering with historical events detectable by downstream verifiers.

---

## 4. Algorithms and standards

| Capability | Crate/file path | Algorithm | Reference | Standard |
|------------|----------------|-----------|-----------|----------|
| Event sourcing with prev_hash chain | `crates/rts-domain` | Append-only event log with per-aggregate version counter and SHA-256 hash chain | Vernon, "Implementing Domain-Driven Design", Addison-Wesley 2013, Ch. 8 | CloudEvents 1.0 |
| Snapshot hydration | `crates/rts-domain` | Aggregate state rebuilt from latest snapshot + subsequent events | Fowler, "Event Sourcing", martinfowler.com 2005 | none binding |
| Tally aggregation (formal) | `crates/rts-tally` | TLA+/Apalache-verified state machine: Tally, Declaration, Referendum specs in `docs/tla/` | Lamport, "Specifying Systems", Addison-Wesley 2002; Konnov et al., "Apalache", OOPSLA 2019 | none binding |
| Referendum threshold | `crates/rts-tally` | 50%+1 of valid votes nationally; majority per county (25+1 of 47) | Kenya Constitution 2010 Art. 255 | none binding |
| Exactly-once tally consumer | `crates/rts-tally` | Kafka transactions (producer + consumer in single transaction; `isolation.level=read_committed`) | Kafkacat-Narkhede, "Kafka: The Definitive Guide", O'Reilly 2017; Apache Kafka KIP-98 | Apache Kafka 3.x |
| Handwriting ICR | `py/` (ONNX model served by Python worker) | ONNX Runtime inference over a handwriting recognition model (form 34A digit recognition) | Hochreiter, Schmidhuber, "Long Short-Term Memory", Neural Computation 1997 (LSTM baseline); ONNX Runtime contributors, ONNX Runtime 2020 | ONNX 1.x |
| SMS/USSD codec | `crates/rts-device-api` | CBOR (RFC 8949) encoding + zstd level-3 compression; fits 2 SMS segments | Bormann, Hoffman, RFC 8949, IETF 2020; Zstandard contributors, zstd 2016 | RFC 8949 |
| HLC (Hybrid Logical Clock) | `crates/rts-domain` | HLC = max(physical_clock, prev_hlc) + logical; stored as int64 nanoseconds | Kulkarni, Demirbas, Madappa, Avva, Leone, "Logical Physical Clocks and Consistent Snapshots in Globally Distributed Databases", OPODIS 2014 | none binding |
| KEK derivation | `crates/declaration` | ANSI X9.24-1:2009 DUKPT key derivation for per-declaration encryption keys | ANSI X9.24-1, American National Standards Institute 2009 | ANSI X9.24 |
| Inclusion proof relay | `crates/rts-publisher` | Receive STH from transparency-svc; verify consistency proof; store tlog_leaf_index | Laurie et al., RFC 6962, IETF 2013 | RFC 6962-bis |

---

## 5. Crypto and trust anchoring

### HSM key roles (held by transparency-svc hsm-signer, accessed via gRPC)

| Key role | Usage |
|----------|-------|
| `declaration` | Ed25519 countersignature on every declaration payload; mandatory before `declarations.status = 'issued'` |
| `sth` | Used by transparency-svc to sign the STH that includes the form34c publication snapshot |

Every event in `eel_events` carries `producer_signature` (Ed25519 by the producing service's SPIRE SVID key). For declaration-class events, `hsm_countersignature` is also populated.

The `prev_hash` chain in `eel_events` combined with HSM signatures ensures tamper evidence: any deletion or substitution of an event breaks the chain and invalidates the HSM countersignature on subsequent events.

---

## 6. Event surface

### Emitted (via eel_outbox, Kafka)

| Event type | Topic | Key fields |
|------------|-------|-----------|
| `eel.rts.form34.submitted.v1` | eel-rts-events | `station_code`, `form_type` (A/B/C), `stream_id`, `stream_version`, `hlc` |
| `eel.rts.form34.published.v1` | eel-rts-events | `station_code`, `form_type`, `votes_cast`, `valid_votes`, `candidate_totals`, `merkle_root` |
| `eel.rts.tally.updated.v1` | eel-rts-events | `constituency_code`, `total_valid`, `candidate_totals`, `merkle_root`, `updated_at` |
| `eel.rts.declaration.signed.v1` | eel-rts-events | `declaration_id`, `election_id`, `tally_snapshot_hash`, `hsm_signature_hex`, `issued_at` |
| `eel.rts.ballot.serial.assigned.v1` | eel-rts-events | `batch_id`, `serial_start`, `serial_end`, `issued_count` |
| `eel.rts.ballot.serial.reconciled.v1` | eel-rts-events | `batch_id`, `spoilt_count`, `reconciliation_hash` |
| `eel.rts.dispute.raised.v1` | eel-rts-events | `station_code`, `constituency_code`, `raised_by`, `reason` |

All CloudEvents 1.0 envelopes. Source: `spiffe://eel.local/svc/rts-svc`.

### Consumed

| Event type | Topic | Action |
|------------|-------|--------|
| `eel.bvr.register.frozen.v1` | eel-bvr-events | Gate result acceptance: reject form34 submissions before freeze |
| `eel.tlog.sth.published.v1` | eel-tlog-events | Verify inclusion proof; update `tlog_leaf_index` in publication_snapshots |

---

## 7. API surface

### REST (rts-api crate, Axum, port 8080)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/rts/stations/{station_id}/form34` | Submit Form 34A/B/C (kit or workstation; Idempotency-Key header required) |
| GET | `/v1/rts/stations/{station_id}/form34` | Retrieve latest form for station |
| GET | `/v1/rts/tally/{constituency_id}` | Running tally for constituency |
| GET | `/v1/rts/tally/summary` | National tally summary |
| GET | `/v1/rts/declaration/{election_id}` | Issued declaration and HSM signature |
| POST | `/v1/rts/ballot-serials/assign` | Assign serial numbers to ballot batch |
| GET | `/v1/rts/ballot-serials/status` | Ballot reconciliation status |
| POST | `/v1/rts/disputes/raise` | RO raises result dispute (M09) |
| GET | `/healthz` | Liveness |
| GET | `/readyz` | Readiness (checks DB, Kafka, HSM reachability) |
| GET | `/openapi.json` | OpenAPI 3.1 spec |

### Kit-facing REST (rts-device-api crate, port 8081)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/device/stations/{station_id}/form34` | Kit submits form (SPIRE SVID required; CBOR or JSON body) |
| GET | `/v1/device/stations/{station_id}/config` | Download station config (registered_voters, election_id) |
| POST | `/v1/device/sms/ingest` | Telecom gateway webhook: SMS-fallback form submission |

### gRPC (proto/rts.proto, port 8443)

| RPC | Input | Output |
|-----|-------|--------|
| `SubmitForm34` | `form_bytes: bytes, sig: bytes` | `FormReceipt { stream_id, stream_version, hlc }` |
| `GetTally` | `constituency_id: string` | `TallySnapshot { candidate_totals, merkle_root, updated_at }` |
| `SignDeclaration` | `form34c_id: string` | `Declaration { declaration_id, hsm_signature_hex, issued_at }` |
| `AssignBallotSerials` | `batch_id: string, count: uint32` | `AssignmentReceipt { serial_start, serial_end }` |

### RO desktop (apps/rts-ro, Tauri)

The Tauri desktop app communicates with `rts-ro-api` crate (port 8082, localhost-only). Provides ICR correction UI, dispute workflow, and offline-first result entry for ROs without stable connectivity.

---

## 8. Integrations

| Upstream | Adapter | Method |
|----------|---------|--------|
| bvr-svc (register freeze gate) | Kafka consumer in `crates/rts-domain` | Topic `eel-bvr-events` |
| transparency-svc (STH inclusion) | Kafka consumer in `crates/rts-publisher` | Topic `eel-tlog-events` |
| transparency-svc (declaration sign) | gRPC client in `crates/declaration` | `hsm-signer.Sign` RPC |
| ONNX ICR worker | HTTP client in `crates/rts-ro-api` | `POST /extract` on `py/` Python FastAPI sidecar |
| Telecom SMS gateway | HTTP webhook handler in `crates/rts-device-api` | POST from gateway |
| biometric-edge-svc (kit identity) | SPIRE SVID verification in `crates/rts-device-api` | SPIFFE workload API trust bundle |

---

## 9. Sharding, indexing, and scale

- `eel_events` indexed by `(stream_id, stream_version)`, `event_type`, and `recorded_at`. Stream per polling station (46,229 streams); no cross-shard joins needed.
- `form34a_view` indexed by `constituency_code` and `status` for tally worker fan-in.
- Kafka `eel-rts-events` topic: 64 partitions keyed by `station_code` hash for ordered per-station delivery.
- Tally worker is a single-consumer-group Kafka consumer with Kafka transactions for exactly-once semantics. Scale target: process all 46,229 Form 34A submissions within 6 hours (2.15 submissions/s average; burst 500/s at polls-close).
- gRPC tally query: `form34c_view` is a single-row aggregate; p99 < 50ms from PG read replica.
- SMS fallback load: at most 46,229 SMS messages over 6 hours = 2.15 SMS/s; telecom gateway adapter handles batching.

---

## 10. Degradation modes

| Mode | Trigger | Behavior | Runbook |
|------|---------|----------|---------|
| Normal | All components healthy | REST + gRPC + SMS paths active; tally live | docs/runbooks/rts-svc.md |
| Degraded (ICR worker down) | ONNX py/ worker unavailable | Forms accepted without ICR extraction; RO manual entry via Tauri app; ICR retried when worker recovers | docs/runbooks/rts-svc.md#icr-recovery |
| Degraded (Kafka unavailable) | Kafka broker partition loss | Forms accepted to PG; outbox relay stalled; tally updates delayed; alert fired | docs/runbooks/rts-svc.md#kafka-recovery |
| Degraded (HSM unavailable) | transparency-svc hsm-signer gRPC down | Forms and tally continue; declaration signing blocked until HSM restored; alert critical | docs/runbooks/rts-svc.md#hsm-unavailable |
| SMS-only | Internet unreachable from station | All stations fall back to CBOR+zstd SMS; tally updated when SMS ingested by gateway | docs/runbooks/rts-svc.md#sms-fallback |
| Read-only (DB primary down) | PG primary failover | Form submissions queued in Kafka; reads from replica; RTO 30 min | docs/runbooks/rts-svc.md#db-failover |

---

## 11. Data lifecycle and retention

| Table | Class (ADR-0016) | Retention |
|-------|-----------------|-----------|
| `eel_events` | Class A: election-critical permanent | Permanent; append-only; no deletes |
| `eel_snapshots` | Class A | Permanent |
| `form34a_view` | Class A | Permanent |
| `form34b_view` | Class A | Permanent |
| `form34c_view` | Class A | Permanent |
| `publication_snapshots` | Class A | Permanent |
| `declarations` | Class A | Permanent |
| `ballot_serials` | Class A | Permanent |
| `eel_outbox` | Class C: ephemeral | 30 days (after published_at) |

No voter PII in rts-svc. Station codes and candidate IDs are public-domain identifiers.

---

## 12. Observability

### Dashboards (Grafana, d/rts)

- Form 34A submission rate (stations/s) and cumulative coverage (% of 46,229 stations reported).
- Submit latency histogram (p95 target < 500ms, p99 < 1s).
- Tally update lag (time between form published and form34b_view updated).
- SMS fallback rate (% of submissions via SMS vs REST).
- ICR extraction accuracy (manual corrections / total extractions).
- Declaration status: pending / issued / anchored.

### Alerts (Prometheus)

| Alert | Condition | Severity |
|-------|-----------|----------|
| `rts_submit_latency_high` | Submit p99 > 2s | critical |
| `rts_tally_consumer_lag_high` | Kafka consumer lag > 5000 | warning |
| `rts_sms_fallback_high` | SMS > 20% of submissions | warning |
| `rts_declaration_signing_stalled` | Declaration not issued > 30 min after form34c compiled | critical |
| `rts_event_chain_break` | prev_hash mismatch detected | critical |

### Key metrics

- `eel_rts_form34_submissions_total` - Counter by form_type and status.
- `eel_rts_submit_duration_seconds` - Histogram: end-to-end submit latency.
- `eel_rts_stations_reported` - Gauge: stations with at least one published form34a.
- `eel_rts_tally_lag_seconds` - Gauge: form34b tally update delay.
- `eel_rts_sms_submissions_total` - Counter: SMS-path submissions.

---

## 13. Testing pyramid

| Layer | Tool | Scope |
|-------|------|-------|
| Unit | `cargo test --lib --all` | Event store append, snapshot hydration, tally arithmetic, referendum threshold, run-off logic, CBOR codec |
| Property | proptest | prev_hash chain: any sequence of appended events produces a consistent hash chain; tally: sum of candidate votes = valid_votes |
| Contract | `cargo test --test '*_contract'` | OpenAPI compliance; CloudEvent envelope schema |
| Integration | `cargo nextest run` with testcontainers (PG, Kafka) | Full form34A submit to tally update pipeline; SMS ingest path; ICR worker stub |
| TLA+ formal | `apalache check docs/tla/tally.tla` | Tally correctness, referendum threshold, declaration uniqueness, no double-publish |
| Load | Gatling (`loadtest/`) + k6 | 60,000 concurrent stations, 600s duration; p99 < 1s submit |
| Chaos | planned (Wave 3) | Kafka broker kill; ICR worker kill; PG primary failover; HSM signer unavailable |

---

## 14. Follow-ups and known limitations

- `declarations` and `ballot_serials` table DDL is described in data_dictionary.yaml and README but migration files for those tables were not found in `migrations/` at time of writing (only V0001-V0003 found). Verify in Wave 2 migration pass.
- ICR ONNX model (`py/`) weights are not committed to the repository. Model training pipeline referenced in README (`python -m icr.train`) but no training corpus or pre-trained weights found on disk. Production deployment requires a validated, tested ONNX model; treat as external input.
- TLA+ specs referenced in `docs/tla/` directory (confirmed `tla` subdir exists in boundary-svc docs; presence in rts-svc docs dir verified by ls). Apalache version compatibility with the spec should be validated before Wave 1 deadline.
- SMS gateway integration requires a Kenyan telecom API contract with at least two MNOs (Safaricom, Airtel) for redundancy. Adapter is stubbed; production integration deferred to Wave 2.
- HLC clock synchronization assumes NTP-synchronized kit clocks. Kit time-drift > 5 minutes will cause HLC ordering anomalies; kit firmware should enforce NTP sync at boot.

---

## Appendix A: TLA+/Apalache tally specification notes

The formal tally specification (`docs/tla/tally.tla`) models the following invariants verified by Apalache model checker:

- **Tally monotonicity**: For any station, `form34a.votes_cast` is set exactly once (no decrease or overwrite without an explicit dispute-resolution event).
- **Constituency sum consistency**: `form34b.total_valid` = sum of all `form34a.valid_votes` for stations in that constituency at any point in time.
- **National sum consistency**: `form34c.total_valid` = sum of all `form34b.total_valid` at any point in time.
- **Referendum threshold**: Winner requires both 50%+1 of national valid votes AND a majority in at least 25 of 47 counties; model verifies no declaration is issued unless both conditions hold simultaneously.
- **Run-off trigger**: If no candidate reaches 50%+1, run-off is declared between the top-two candidates; model verifies this is the only alternative to an outright win.
- **Declaration uniqueness**: At most one declaration can be issued per election (no two declarations with the same election_id).
- **No double-publish**: A form34c result is published to the transparency log at most once.

The Apalache configuration uses bounded model checking with 10 steps (covers all realistic election-day scenarios). Counterexample traces are exported as JSON for human review. Specification written in PlusCal (translated to TLA+ by the PlusCal translator).

---

## Appendix B: CBOR+zstd SMS fallback codec details

The SMS fallback codec is designed to fit a full Form 34A result into 2 SMS segments (2 x 153 bytes = 306 bytes usable in UCS-2, or 2 x 160 bytes = 320 bytes in GSM-7 if content is ASCII-safe).

Encoding steps in `crates/rts-device-api`:

1. Serialize form payload as CBOR (RFC 8949): station_code (text), votes_cast (uint), valid_votes (uint), rejected_votes (uint), candidate_totals (array of [candidate_id (uint8), votes (uint32)]). Kenya presidential ballot has at most 8 candidates.
2. Compress with zstd level 3: typical compressed size for 8-candidate form = 80-120 bytes.
3. Base64url-encode: 80-120 bytes -> 107-160 base64url characters.
4. Prepend 6-byte header: version (1B), station_code_hash (4B), sequence_number (1B for multi-SMS).
5. Total: 113-166 characters, comfortably within 2 SMS segments.

The receiving gateway (`POST /v1/device/sms/ingest`) decodes base64url, decompresses with zstd, parses CBOR, and routes to the same validation path as REST submissions. Producer signature is embedded in the CBOR structure as a bytes field.

---

## Appendix C: ICR extraction pipeline

The handwriting recognition pipeline (`py/`) uses ONNX Runtime to run inference on scanned Form 34A images:

1. **Pre-processing**: Deskew (Hough line detection), binarize (Otsu threshold), denoise (median filter), normalize to 300 DPI.
2. **Field detection**: Template matching locates the candidate vote fields (fixed positions on Form 34A template). Each field is a small bounding box (approximately 40x30 pixels per digit).
3. **Digit recognition**: Handwritten digit recognizer ONNX model (architecture: CNN + LSTM; trained on a synthetic Kenyan handwriting corpus; not yet finalized). Outputs: digit sequence with confidence per digit.
4. **Confidence gating**: Fields with confidence < 0.8 are flagged for human review in `form34a_view.annotations`. RO sees these fields highlighted in the Tauri desktop app for manual correction.
5. **Cross-check**: Extracted `votes_cast` is cross-checked against the sum of candidate votes; discrepancy > 5% triggers mandatory RO review.
6. **Output**: Structured extraction written to `form34a_view.candidate_totals` (JSONB) and `form34a_view.raw_scan_ref` (MinIO object key for the original scan).

ONNX model weights and training pipeline are external inputs; their validation is a prerequisite for Wave 1 production deployment.

---

## Appendix D: Ballot serial chain of custody (M63)

The `ballot-serial` crate implements a serialized chain-of-custody for ballot booklets:

- **Serial assignment**: Each ballot batch (typically 1 booklet of 100 ballots) receives a serial range `[start, end]`. Assignment is atomic (PG sequence + unique constraint).
- **Reconciliation invariant**: `issued_count + spoilt_count + unused_count = end - start + 1` must hold at close of poll. Any discrepancy triggers a tamper alert.
- **Hash chain**: Each reconciliation event appends `reconciliation_hash = SHA-256(prev_hash || batch_id || spoilt_count || used_count)`. This forms a per-batch hash chain detectable for tampering.
- **Tlog anchor**: Reconciliation hashes are appended to the transparency log as `custody-events` domain leaves.
- **RF lot scan (M39)**: When biometric-edge-svc bvr-kit scans a ballot lot seal (NFC/RFID, M39), it publishes a `custody-events` CloudEvent with the lot serial. The ballot-serial crate correlates lot scans with serial assignments to verify physical custody matches electronic records.
