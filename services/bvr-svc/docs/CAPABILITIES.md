# bvr-svc capabilities

Authoritative voter register for 22 million Kenyans: county-partitioned event-sourced voter records with field-encrypted PII, audit-shadow triggers, Merkle-root snapshots, IPRS fail-closed lookups, and ODPC-compliant deregistration appeals.

**Tier**: T1-election-critical.
**SLO**: 99.99% availability. Ingest 1,000 records/s; 30M register generation in < 30 min; full export < 4 hours.
**RTO**: 4 hours. **RPO**: 15 min.

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
| M02 | bvr-backend (voter register) | WP-3.1 |
| M35 | CVR continuous voter register | WP-3.5 |
| M36 | Voter deregistration and appeals | WP-3.6 |
| M53 | Voter ID card (scope-gated) | WP-3.9 |

### Timeline role

| Phase | Activity |
|-------|----------|
| T-24 months | Baseline register loaded from CRS/NRB; biometric dedup via ABIS begins |
| T-18 months | Continuous register updates live; §6A public inspection endpoint open |
| T-12 months | Deregistration appeals window opens; ODPC-compliant erasure enforced |
| T-6 months | M53 voter ID card production pipeline starts (scope-gated) |
| T-30 days | Register snapshot published; Merkle root anchored to transparency log |
| T-3 days | bvr.register.frozen event emitted; RTS result acceptance window opens |
| Election day | Read-only; voter lookup served from edge read replicas |
| T+90 days | ODPC annual erasure sweep; deregistered records purged from encrypted storage |

---

## 2. End-to-end pipelines

### 2.1 Batch ingest (CRS/NRB/KRA delta)

```
CRS/NRB/KRA delta feed -> bvr-batch crate (streaming ingest worker)
  -> parse row -> NFC normalize (Unicode NFC) name fields
  -> IPRS lookup (fail-closed: if IPRS unreachable, reject row, do not silently accept)
  -> check for canary row collision (reject if canary slot)
  -> encrypt PII fields (national_id, full_name, dob, phone) with per-county DEK
  -> upsert voters table (county partition) -> increment version counter
  -> audit_voters_trigger fires: insert into audit_voters_history
  -> checkpoint progress to ingest_checkpoints table (per-row resumable)
  -> per-row report: accepted / rejected / skipped (written to MinIO batch report)
```

### 2.2 Voter lookup

```
client GET /v1/bvr/voters/{voter_id} or POST /v1/bvr/voters/lookup
  -> bvr-api crate
  -> route to county partition (voters_{county_code})
  -> decrypt PII fields for authorized caller (SPIRE SVID scope check)
  -> return VoterRecord (excluding biometric template refs for non-ABIS callers)
```

### 2.3 Register snapshot and Merkle root

```
bvr-domain crate (scheduled nightly or on-demand)
  -> SELECT voter_id, version, county_code, biometric_template_refs FROM voters
     ORDER BY voter_id  (deterministic ordering)
  -> compute Merkle tree over SHA-256(row_canonical_json) leaf hashes
  -> insert into register_snapshots (snapshot_date, tree_size, merkle_root, citizen_count)
  -> publish bvr.register.snapshot.published CloudEvent
  -> transparency-svc appends snapshot hash to log
```

### 2.4 Register freeze

```
Commission trigger (admin API) -> bvr-domain: freeze(election_id)
  -> set register_snapshots.status = 'frozen' for current snapshot
  -> publish bvr.register.frozen CloudEvent
  -> rts-svc receives event: result acceptance window opens
  -> bvi-svc receives event: register extract distribution begins
```

### 2.5 Deregistration appeal (M36)

```
appellant POST /v1/bvr/deregistration-appeals
  -> dereg-appeal crate: validate grounds (death, double-enrolment, court order, ODPC right-to-erasure)
  -> create appeal case -> notify ODPC if erasure-class
  -> on approval: set voters.status = 'deregistered'
  -> audit_voters_trigger fires: final history row with operation='DELETE'
  -> if ODPC erasure: overwrite encrypted PII fields with zeros, then delete row
  -> publish bvr.voter.deregistered CloudEvent
```

### 2.6 §6A public inspection (M35)

```
citizen GET /v1/bvr/inspection/roll
  -> cvr crate: aggregate voter count by constituency (anonymized, no PII)
  -> return InspectionRoll { constituency_code, voter_count, snapshot_date }
  -> rate-limited per /64 subnet to prevent enumeration
```

---

## 3. Domain aggregates and state stores

Tables from `migrations/V1__create_voters.sql`, `V2__audit_shadow.sql`, `V3__snapshots.sql`, `V4__dedup_and_sar.sql`, `V5__canary_reads_and_outbox.sql`, `V6__rls_policies.sql`.

| Table | Purpose |
|-------|---------|
| `voters` | Core voter register. Columns: `voter_id` (TEXT, UUID7-format), `version` (BIGINT), `county_code` (CHAR(3), partition key), `national_id_enc` (BYTEA, field-encrypted), `full_name_enc` (BYTEA), `dob_enc` (BYTEA), `phone_enc` (BYTEA nullable), `polling_station_id`, `election_scope`, `status` (pending/active/dead/invalid/deregistered), `biometric_template_refs` (JSONB), `is_canary` (BOOLEAN), `key_id`, `created_at`, `updated_at`. Partitioned by LIST(`county_code`), 47 partitions (001-047). Primary key `(voter_id, county_code)`. |
| `audit_voters_history` | Immutable audit shadow. Columns: `history_id` (UUID), `voter_id`, `county_code`, `version_from`, `version_to`, `operation` (INSERT/UPDATE/DELETE), `changed_by`, `changed_at`, `row_snapshot` (JSONB). Populated by trigger on every voter mutation. |
| `register_snapshots` | Periodic register snapshots with Merkle root. Columns: `snapshot_id`, `snapshot_date`, `tree_size`, `merkle_root`, `citizen_count`, `change_count`, `status` (draft/frozen/anchored). |
| `ingest_checkpoints` (bvr-batch crate) | Resumable batch ingest progress. Planned columns: `batch_id`, `source`, `last_processed_row`, `accepted_count`, `rejected_count`, `checkpoint_at`. Verify in Wave 2. |
| `deregistration_cases` (dereg-appeal crate) | Appeal case management. Planned columns: `case_id`, `voter_id`, `grounds`, `status`, `odpc_ref`, `decided_at`, `decided_by`. Verify in Wave 2. |
| `biometric_template_refs` | References only (no template blobs in bvr-svc); blobs stored and encrypted by abis-svc. JSONB array in `voters.biometric_template_refs`. |

The `is_canary` flag marks synthetic rows inserted for silent-corruption detection. Canary rows are spread across county partitions; periodic reads verify they are unchanged and alert on unexpected modifications.

---

## 4. Algorithms and standards

| Capability | Crate/file path | Algorithm | Reference | Standard |
|------------|----------------|-----------|-----------|----------|
| County-partitioned voter store | `crates/bvr-domain` | PostgreSQL LIST partitioning by county_code (47 partitions 001-047) | PostgreSQL 17 Table Partitioning, postgresql.org 2024 | none binding |
| Versioned voter aggregate | `crates/bvr-domain` | Optimistic version counter; prev version required on update to prevent lost update | Kleppmann, "Designing Data-Intensive Applications", O'Reilly 2017 Ch. 7 | none binding |
| Audit-shadow trigger | `migrations/V2__audit_shadow.sql` | Append-only history table populated by PostgreSQL AFTER trigger; operation = TG_OP | NIST SP 800-92 "Guide to Computer Security Log Management" | ISO/IEC 27001 A.12.4 |
| Merkle root over snapshot | `crates/bvr-domain` | SHA-256 leaf hashes over row canonical JSON; binary Merkle tree root; deterministic ordering by voter_id | Merkle, "A Digital Signature Based on a Conventional Encryption Function", CRYPTO 1987 | none binding |
| Field-level PII encryption | `crates/bvr-domain` (via eel-crypto-rs) | AES-256-GCM with per-county DEK, KEK-wrapped (eel-crypto-rs crate) | NIST SP 800-38D, "Galois/Counter Mode", 2007 | ISO/IEC 27001 A.10, Kenya Data Protection Act 2019 §25 |
| NFC normalization | `crates/bvr-batch` | Unicode Normalization Form C (NFC) on name fields before storage and hash | Unicode Consortium, "Unicode Standard Annex #15", 2023 | none binding |
| IPRS fail-closed lookup | `crates/bvr-batch` | HTTP client to IPRS API; on error: reject row, log rejection, do not silently accept | Fail-safe default pattern; Kenya National Integrated Identity Management System Act 2019 | none binding |
| Canary row detection | `crates/bvr-domain` | Synthetic rows with known values at fixed voter_id positions; periodic read probe; alert on mismatch | "Canary Values for Integrity Monitoring", NIST SP 800-53 SI-7 | none binding |
| Streaming batch ingest with checkpoint | `crates/bvr-batch` | Per-row checkpoint to ingest_checkpoints; on resume: skip rows before last_processed_row | Kleppmann, "Designing Data-Intensive Applications", O'Reilly 2017 Ch. 11 | none binding |
| RLS policies | `migrations/V6__rls_policies.sql` | PostgreSQL Row-Level Security; role-based access to county partitions | PostgreSQL 17 Row Security Policies | ISO/IEC 27001 A.9.4 |

---

## 5. Crypto and trust anchoring

PII fields (`national_id_enc`, `full_name_enc`, `dob_enc`, `phone_enc`) are encrypted with AES-256-GCM per-county DEKs. DEKs are wrapped under the per-election-cycle KEK managed by transparency-svc's HSM key ceremony. `voters.key_id` identifies which KEK generation was used for each row, enabling key rotation without decrypting all rows simultaneously.

Register snapshot Merkle roots are appended to the transparency log via `bvr.register.snapshot.published` events, giving independent verifiers a tamper-evident anchor to check register integrity against.

`audit_voters_history` rows are append-only (no UPDATE/DELETE permitted; enforced by trigger returning NEW, never modifying OLD). This provides a forensic record for ODPC audit and election court proceedings.

---

## 6. Event surface

### Emitted (via outbox, Kafka)

| Event type | Topic | Key fields |
|------------|-------|-----------|
| `eel.bvr.voter.enrolled.v1` | eel-bvr-events | `voter_id`, `county_code`, `polling_station_id`, `enrolled_at` |
| `eel.bvr.voter.updated.v1` | eel-bvr-events | `voter_id`, `county_code`, `version`, `changed_fields` |
| `eel.bvr.voter.deregistered.v1` | eel-bvr-events | `voter_id`, `county_code`, `grounds`, `decided_at` |
| `eel.bvr.register.frozen.v1` | eel-bvr-events | `election_id`, `snapshot_id`, `merkle_root`, `frozen_at` |
| `eel.bvr.register.snapshot.published.v1` | eel-bvr-events | `snapshot_id`, `snapshot_date`, `tree_size`, `merkle_root`, `citizen_count` |
| `eel.bvr.deregistration.appealed.v1` | eel-bvr-events | `case_id`, `voter_id`, `grounds`, `submitted_at` |

All CloudEvents 1.0. Source: `spiffe://eel.local/svc/bvr-svc`.

### Consumed

| Event type | Topic | Action |
|------------|-------|--------|
| `eel.abis.match.completed.v1` | eel-abis-events | Update `biometric_template_refs` with ABIS subject_id on successful dedup |
| `eel.biometric-edge.enrolment.submitted.v1` | eel-edge-enrolment | Trigger ABIS dedup job for new field-captured biometric |

---

## 7. API surface

### REST (bvr-api crate, Axum, port 8080)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/bvr/voters/{voter_id}` | Voter lookup by ID (SPIRE SVID required; PII decrypted for authorized scope) |
| POST | `/v1/bvr/voters/lookup` | Batch voter lookup (name, DOB, ID number); returns matched voter IDs |
| GET | `/v1/bvr/register/snapshot` | Current register snapshot (tree_size, merkle_root, citizen_count) |
| GET | `/v1/bvr/register/export` | Export full register (chunked GeoJSON + CSV; auth required) |
| GET | `/v1/bvr/inspection/roll` | §6A public inspection: voter count per constituency (anonymized) |
| POST | `/v1/bvr/deregistration-appeals` | File deregistration appeal |
| GET | `/v1/bvr/deregistration-appeals/{case_id}` | Appeal status |
| POST | `/v1/bvr/admin/freeze` | Freeze register for election (Commission admin role required) |
| GET | `/healthz` | Liveness |
| GET | `/readyz` | Readiness (checks DB, IPRS connectivity) |
| GET | `/openapi.json` | OpenAPI 3.1 spec |

### gRPC (proto/bvr.proto, port 8443)

| RPC | Input | Output |
|-----|-------|--------|
| `LookupVoter` | `voter_id: string` | `Voter { voter_id, county_code, polling_station_id, status, biometric_template_refs }` |
| `BatchLookup` | stream of `LookupRequest` | stream of `LookupResult { voter_id, matched: bool, status }` |
| `GetRegisterSnapshot` | `constraint: SnapshotConstraint` | `RegisterSnapshot { snapshot_id, merkle_root, tree_size, citizen_count }` |
| `PublishInspectionRoll` | `period: string` | `InspectionRoll { constituency_counts: map }` |
| `FileDeregistrationAppeal` | `request: AppealRequest` | `CaseId { case_id: string }` |

---

## 8. Integrations

| Upstream | Adapter | Method |
|----------|---------|--------|
| IPRS (identity verification) | HTTP client in `crates/bvr-batch` | REST API; fail-closed on error |
| CRS/NRB/KRA (data feeds) | `crates/bvr-batch` batch worker | SFTP/HTTP delta feeds; scheduled |
| abis-svc (biometric dedup) | Kafka consumer `eel-abis-events` + gRPC client | abis.proto `KitVerify` for 1:1; `MatchOneToN` for dedup |
| transparency-svc (snapshot anchoring) | Kafka outbox relay | `eel.bvr.register.snapshot.published.v1` |
| OpenBao (DEK storage) | `crates/bvr-domain` via eel-crypto-rs | OpenBao KV v2 API |

---

## 9. Sharding, indexing, and scale

- 47 county partitions (`voters_001` through `voters_047`) via PostgreSQL LIST partitioning. Partition pruning reduces scan scope for county-level operations (e.g., register export by county).
- Indexes: `status` partial index (WHERE status != 'deregistered'); `election_scope, county_code` compound; `is_canary` partial (WHERE is_canary = TRUE); `updated_at DESC`.
- Read replicas in Nairobi, Mombasa, Kisumu, Eldoret, Garissa for low-latency voter lookup during election day.
- Ingest target: 1,000 records/s; 22M records in 6.1 hours; 30M (full register generation) in < 30 min via parallel county-partition writers.
- Export target: full 22M register export < 4 hours (chunked streaming via GeoJSON + CSV); parallelized by county partition.
- `audit_voters_history` grows at audit-event rate; monthly partitioning recommended in Wave 2.

---

## 10. Degradation modes

| Mode | Trigger | Behavior | Runbook |
|------|---------|----------|---------|
| Normal | All components healthy | Ingest + lookup + snapshots at full capacity | docs/runbooks/bvr-svc.md |
| Degraded (IPRS unavailable) | IPRS HTTP timeout or error | Batch ingest rejects all new rows (fail-closed); existing register unchanged; alert fired; resume when IPRS restored | docs/runbooks/bvr-svc.md#iprs-unavailable |
| Degraded (ABIS unavailable) | ABIS gRPC unreachable | New enrolments accepted and queued; dedup held; existing register unaffected | docs/runbooks/bvr-svc.md#abis-unavailable |
| Read-only (election day) | register.frozen event received | No writes permitted; all reads from replicas; export disabled | docs/runbooks/bvr-svc.md#election-day |
| Read-only (DB primary down) | PG primary failover | Lookups from replicas; writes queued; RTO 4 hours | docs/runbooks/bvr-svc.md#db-failover |
| Canary alert | Canary row modified unexpectedly | PagerDuty critical; all writes suspended until investigation complete | docs/runbooks/bvr-svc.md#canary-alert |

---

## 11. Data lifecycle and retention

| Table | Class (ADR-0016) | Retention |
|-------|-----------------|-----------|
| `voters` | Class A: election-critical | Permanent for active voters; deregistered rows zeroed and deleted per ODPC schedule |
| `audit_voters_history` | Class A | Permanent (forensic record) |
| `register_snapshots` | Class A | Permanent |
| `deregistration_cases` | Class B: operational | 10 years |
| `ingest_checkpoints` | Class C: ephemeral | 90 days |
| Outbox | Class C | 30 days |

Kenya Data Protection Act 2019 §25 mandates that personally identifiable information retained beyond its original purpose is unlawful. ODPC erasure requests processed within 30 days via dereg-appeal crate.

---

## 12. Observability

### Dashboards (Grafana, d/bvr)

- Register metrics: voter count by county, change rate, ingest rate, export rate.
- Lookup performance: p50/p95/p99 latency, cache hit rate, batch size distribution.
- Biometric dedup: ABIS match rate, dedup latency, false-match rate.
- Inspection metrics: §6A request rate, roll generation time.
- Canary health: last probe time, canary row status by partition.

### Alerts (Prometheus)

| Alert | Condition | Severity |
|-------|-----------|----------|
| `bvr_ingest_lag_high` | Ingest worker backlog > 10,000 rows | warning |
| `bvr_iprs_unavailable` | IPRS health check fails | critical |
| `bvr_canary_mismatch` | Canary row value differs from expected | critical |
| `bvr_export_duration_high` | Full export takes > 4 hours | warning |
| `bvr_write_during_freeze` | Write attempted after register.frozen | critical |

### Key metrics

- `eel_bvr_voters_total` - Gauge by county and status.
- `eel_bvr_ingest_rate_records_per_second` - Gauge: current ingest throughput.
- `eel_bvr_lookup_duration_seconds` - Histogram by lookup type.
- `eel_bvr_merkle_root_computed_at` - Gauge: last snapshot timestamp.
- `eel_bvr_canary_probe_success` - Gauge: 1 = healthy, 0 = mismatch.

---

## 13. Testing pyramid

| Layer | Tool | Scope |
|-------|------|-------|
| Unit | `cargo test --lib --all` | NFC normalization, field encryption/decryption, Merkle root computation, canary row detection |
| Property | proptest | Round-trip: encrypt then decrypt any byte sequence produces original; Merkle root is deterministic for same voter set |
| Contract | OpenAPI conformance | REST response shapes; gRPC proto compatibility |
| Integration | `cargo nextest run` with testcontainers (PG partitioned) | Ingest 1k records/s for 60s; audit trigger fires on every mutation; RLS blocks cross-county read |
| Scale | `cargo test ingest_1k_per_sec --release`; k6 (`loadtest/register-export.js`) | 1,000 records/s sustained ingest; 30M voter export in < 30 min |
| Chaos | planned (Wave 3) | IPRS unavailable (verify fail-closed); ABIS unavailable; PG primary failover |
| Compliance | ODPC erasure test | File appeal; verify PII zeroed and row deleted; verify audit history preserved |

---

## 14. Follow-ups and known limitations

- `ingest_checkpoints` and `deregistration_cases` tables referenced in README and data_dictionary.yaml but migration DDL not found in V1-V6 migrations at time of writing. Verify in Wave 2 migration pass; may be in `crates/bvr-batch` or `crates/dereg-appeal` local migration directories.
- M53 voter ID card (`crates/voter-card`) is scope-gated. The crate directory exists but no business logic was verified on disk. Marked deferred until scope gate is lifted.
- IPRS API authentication credentials and rate-limit parameters need to be confirmed with NIIMS for production deployment. The fail-closed policy is implemented; the credential rotation procedure is not.
- Canary row IDs and expected values should be generated at key ceremony time and stored in OpenBao (not hard-coded). Current implementation details in `crates/bvr-domain` not verified.
- Monthly partitioning of `audit_voters_history` is recommended once the table grows past 50M rows (at 22M voters with typical 3 updates per voter lifetime). Plan this in Wave 2 infrastructure pass.

---

## Appendix A: County partition scheme

The `voters` table is partitioned by LIST(`county_code`) into 47 sub-tables (`voters_001` through `voters_047`), one per Kenyan county. This gives:

| Benefit | Mechanism |
|---------|-----------|
| Partition pruning | Queries with `WHERE county_code = '001'` touch only `voters_001`; PG planner eliminates 46 other partitions |
| Parallel ingest | County-specific batch workers write to separate partitions concurrently without lock contention |
| Selective backup | Each county partition can be backed up independently; county-level restore in < 30 min |
| RLS alignment | PostgreSQL Row-Level Security policies grant election officials access only to their county partition |
| Export parallelism | Full-register export parallelized by county; 47 concurrent COPY workers achieve < 30 min for 22M rows |

County code assignment follows the KNBS Kenya National Bureau of Statistics county number scheme (Nairobi = 047, Mombasa = 001, etc.). The `county_code CHAR(3)` field is zero-padded (e.g., `'001'`, `'012'`, `'047'`).

### Partition maintenance

New partitions are not needed mid-cycle (Kenya has 47 counties, fixed by Constitution). The `DO $$ ... $$` loop in `V1__create_voters.sql` creates all 47 partitions at migration time. The partition table (`voters`) itself is never queried directly by application code; queries target the partition table and rely on PG partition pruning.

Index maintenance: each partition has its own index storage. The global `idx_voters_status`, `idx_voters_election`, `idx_voters_is_canary`, `idx_voters_updated` are declared on the parent table and automatically inherited by all partitions.

---

## Appendix B: Field encryption key management

All Tier A PII columns (`national_id_enc`, `full_name_enc`, `dob_enc`, `phone_enc`) are encrypted per-row with AES-256-GCM before storage. The encryption key hierarchy:

```
KEK (per election cycle, held by transparency-svc HSM, M-of-N keyholder ceremony)
  -> per-county DEK (one per county code, rotated at each election cycle)
     -> per-row GCM nonce (12 bytes, random, prepended to ciphertext)
```

`voters.key_id` stores the KEK generation identifier (e.g., `"kek-2027-v1"`). This allows the system to:
- Re-wrap DEKs under a new KEK without decrypting the rows (DEK-level rewrap via HSM).
- Support multiple concurrent KEK generations during an overlap window (e.g., when a cycle KEK is rotated mid-enrolment).
- Audit which rows use which KEK generation.

Decryption is performed only by bvr-api on authorized API requests. abis-svc receives biometric template references (not raw PII) and operates on encrypted templates. bvi-svc and rts-svc receive only `voter_id` and `polling_station_id` (Tier B operational data, not encrypted at field level).

---

## Appendix C: Audit shadow implementation details

The `audit_voters_trigger_fn()` PL/pgSQL function fires on every INSERT, UPDATE, or DELETE on `voters` (and each partition via trigger inheritance). It records:

- `voter_id`, `county_code`: which record changed.
- `version_from`: the OLD.version at time of change.
- `operation`: TG_OP (INSERT/UPDATE/DELETE).
- `changed_by`: `current_setting('app.current_user', TRUE)` - the application-level user ID set by the API layer via `SET LOCAL app.current_user = '<user_id>'` within each transaction.
- `row_snapshot`: `row_to_json(OLD)::JSONB` - full previous state, including encrypted PII blobs (encrypted at rest; no plaintext in audit log).

The `changed_by` field requires the API layer to set `app.current_user` before any write. If not set (e.g., during a migration), the field is NULL and logged as an anomaly.

The audit history table is append-only by design (no UPDATE or DELETE triggers defined on it; no RLS policies permitting writes from application roles). Periodic integrity verification compares `voters.version` against the count of `audit_voters_history` rows for the same voter_id to detect any gaps.

---

## Appendix D: Canary row design

Canary rows are synthetic voter records inserted at registration time with `is_canary=TRUE`. They have:
- Deterministic `voter_id` values derived from a per-election secret seed (stored in OpenBao).
- Known encrypted PII blobs: the DEK for the canary's county is used to encrypt known plaintexts; periodic probes decrypt and verify against expected values.
- Distribution: at least 3 canary rows per county partition (141 total); their positions chosen to test different index paths (first, last, and middle of partition B-tree).

A canary probe runs every 5 minutes:
1. SELECT canary rows (partial index `WHERE is_canary=TRUE`).
2. Decrypt PII blobs; compare against expected plaintext.
3. Check that `version` has not changed unexpectedly.
4. If any mismatch: fire `bvr_canary_mismatch` Prometheus alert (severity: critical); suspend all writes via circuit breaker until investigation.

Canary rows serve dual purpose: silent corruption detection (storage-layer bit rot) and tamper detection (unauthorized direct DB access bypassing the audit trigger).
