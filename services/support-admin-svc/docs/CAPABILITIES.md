# support-admin-svc capabilities and algorithms

Comprehensive capability catalogue for the administrative backbone service (modules M25, M27, M54, M55, M69). Every capability lists the algorithm, the source reference (paper, standard, or statutory instrument), and the crate or file that implements it.

## Contents

1. Module scope
2. Special voter facility FSM (M25)
3. IDP double-cast guard and s.44A dual sign-off (M25)
4. IAB 5% Merkle-seed sample (M25)
5. Intent preservation: image deletion and adjudication catalogue (M27)
6. Intent z-score outlier detection (M27)
7. Budget segregation of duties (M54)
8. Payroll engine: 250 k staff in 4 h with training gate (M55)
9. M-Pesa B2C batched disbursement (M55)
10. BagIt archive packaging with SHA-256 attestation (M69)
11. Key algorithms and references
12. Integration points
13. Standards and compliance (BagIt RFC 8493, OAIS ISO 14721)
14. Operational excellence
15. Validation harnesses
16. Data lifecycle, API surface, and events

---

## 1. Module scope

| Module | Name | Crate | Key tables |
|---|---|---|---|
| M25 | Special Voter Workflows | crates/special-voter | eel_special_voter_facilities, eel_special_voter_events, eel_special_form34, eel_idp_reassignments, eel_s44a_overrides |
| M27 | Intent Preservation | crates/intent | eel_premark_validations, eel_spoilt_ballot_events, eel_adjudication_events, eel_rejection_rate_alerts, eel_adjudication_toggle_audit |
| M54 | Budget and Expenditure | crates/budget | budget_lines, payment_vouchers |
| M55 | HR and Payroll | crates/payroll | staff_contracts, payroll_runs, payroll_disbursements |
| M69 | Long-Term Archive | crates/archive, py/bagit | archive_collections, ingest_jobs, preservation_attestations |

Tier: T2. Availability SLO: 99.9%. Payroll run SLO: 250,000 staff processed in < 4 h. Archive ingest SLO: > 1 GB/min sustained. RTO: 4 h. RPO: 1 h.

State stores: PostgreSQL 17 (schema-per-module), MinIO (archive object store), Valkey (payroll idempotency cache).

---

## 2. Special voter facility FSM (M25)

Implemented in `crates/special-voter/src/facility.rs`.

### 2.1 Facility categories

`eel_special_voter_facilities.category IN ('REMAND_PRISON','HOSPITAL','BARRACKS','REFUGEE_CAMP','IDP_CAMP')`. Each category has a distinct voting procedure:

- `REMAND_PRISON`: dual-biometric open required (see section 3).
- `BARRACKS`: dual-biometric open required (see section 3).
- `REFUGEE_CAMP`: `REFUGEE_VOTING_ENABLED` feature flag must be set in the deployment ConfigMap; if the flag is `false`, the facility is blocked from transitioning out of `SEALED`.
- `HOSPITAL`, `IDP_CAMP`: standard presiding officer open.

### 2.2 Session state machine

```
SEALED
  -> DUAL_BIOMETRIC_PENDING   (for REMAND_PRISON and BARRACKS)
  -> OPEN                      (session active; voting allowed)
  -> SUSPENDED                 (temporary hold; no voting)
  -> CLOSED                    (session ended; tallies frozen)
```

For `REMAND_PRISON` and `BARRACKS`, the transition from `SEALED` to `OPEN` requires both the presiding officer and the facility commander to authenticate with biometric verification (dual-biometric open). The facility holds in `DUAL_BIOMETRIC_PENDING` until both biometric events are recorded in `eel_special_voter_events` with `event_type: DUAL_BIOMETRIC_AUTH` and distinct `officer_id` values.

### 2.3 Form 34A variants

`eel_special_form34.form_variant IN ('34A-R','34A-H','34A-B','34A-I')` correspond to Remand, Hospital, Barracks, and IDP/Refugee variants respectively. Each variant carries different signature requirements per the Elections (General) Regulations 2012.

---

## 3. IDP double-cast guard and s.44A dual sign-off (M25)

### 3.1 IDP double-cast guard

A voter reassigned to an IDP camp via `eel_idp_reassignments` retains their original polling station registration. To prevent double casting, the system enforces:

- On biometric identification at the IDP facility: a cross-check query to bvr-svc (`GET /v1/bvr/voters/{voter_id}/stations`) confirms the voter has not yet voted at their original station.
- If the cross-check returns `voted: true`, the identification event is blocked and an `IDP_DOUBLE_CAST_DETECTED` event is written to `eel_special_voter_events`.

The cross-check is performed within the same serialisable transaction as the identification event to prevent a race where two concurrent identification attempts at different facilities both pass.

### 3.2 Section 44A dual sign-off

`eel_s44a_overrides` records cases where an override to the normal identification procedure is invoked under Elections Act s.44A. Each override requires:
- `requesting_officer_id`: the presiding officer who requested the override.
- `countersigning_officer_id`: a second officer (must differ from the requesting officer; enforced by `CHECK (requesting_officer_id != countersigning_officer_id)`).
- `approved: bool`: the outcome.

An `approved: true` record is required before the override takes effect. The `reason_code` is drawn from a fixed vocabulary defined in the Elections Act s.44A regulations.

---

## 4. IAB 5% Merkle-seed sample (M25)

### 4.1 Purpose

The Independent Audit Board (IAB) requires a 5 % random sample of special-voter identification events to be independently audited. The sample is drawn using a Merkle-seed approach to provide a verifiable, tamper-evident selection.

### 4.2 Algorithm

Implemented in `crates/special-voter/src/iab_sample.rs`:

1. At the close of each facility session (`CLOSED` state), all `eel_special_voter_events` rows for the facility are hashed individually: `h_i = sha256(event_id || voter_id_enc || officer_id || event_at)`.
2. A Merkle tree is built over the sorted `h_i` values using SHA-256 as the hash function (RFC 6962: Certificate Transparency, Laurie et al., IETF, 2013).
3. The Merkle root is stored in `eel_special_voter_events.merkle_node_id` at the facility level (the root hash).
4. A pseudo-random 5 % sample index list is derived from `blake3(merkle_root || iab_card_value)` where `iab_card_value` is the IAB card entropy provided at session close.
5. The selected events are exported to the IAB audit endpoint (`GET /v1/support/special-voters/{facility_id}/iab-sample`) with Merkle inclusion proofs for each selected event.

The IAB can independently verify each inclusion proof without access to the full event list.

---

## 5. Intent preservation: image deletion and adjudication catalogue (M27)

### 5.1 Pre-mark validation and image deletion

`eel_premark_validations` records the outcome of ballot image inference (`inference_outcome`, `model_version`, `device_id`) with a hard constraint: `image_delete_elapsed_ms <= 1000` (PG CHECK). The inference model runs on the field device; the raw ballot image is deleted within 1 second of inference. The elapsed time (in milliseconds) from inference completion to confirmed image deletion is measured and recorded. Any device that reports `image_delete_elapsed_ms > 1000` is flagged for immediate investigation.

### 5.2 Adjudication catalogue

Spoilt ballot adjudications are constrained to the catalogue in `eel_adjudication_events.reason_code IN ('A1','A2','A3','A4','A5','A6','A7','A8','V1','V2','V3')`:

| Code | Meaning |
|---|---|
| A1 | Unmarked ballot (no mark anywhere) |
| A2 | Multiple marks (more than one candidate marked) |
| A3 | Mark outside designated area |
| A4 | Identification mark (voter identified) |
| A5 | Torn or mutilated beyond interpretation |
| A6 | Official mark missing |
| A7 | Mark equivocal between two candidates |
| A8 | Other (must specify in evidence photo) |
| V1 | Valid: clear mark for one candidate |
| V2 | Valid: mark adjacent to name counts |
| V3 | Valid: other valid mark (commission ruling) |

Photo evidence (`photo_evidence_ref`: MinIO reference) is required for codes A2 through A7.

### 5.3 Decision outcome

`eel_adjudication_events.decision IN ('REJECT','ACCEPT')`. The adjudicator selects a reason code and decision; the pair is validated by the API layer before persistence.

---

## 6. Intent z-score outlier detection (M27)

Implemented in `crates/intent/src/outlier.rs`.

At the end of each counting session, the system computes the rejection rate for each station:

```
rejection_rate = COUNT(decision='REJECT') / COUNT(*) FROM eel_adjudication_events
WHERE station_code = $1 AND adjudicated_at BETWEEN $start AND $end
```

The national mean (`national_mean`) and standard deviation are computed across all stations for the same contest. A z-score is derived:

```
z_score = (rejection_rate - national_mean) / national_std_dev
```

When `|z_score| > 3.0`, an alert row is inserted into `eel_rejection_rate_alerts` (`rejection_rate`, `national_mean`, `z_score`, `acknowledged: false`). Alerts are acknowledged by a senior officer (`acknowledged_by`, `acknowledged: true`).

The z-score threshold of 3.0 corresponds to a one-in-370 false-positive rate under a normal distribution, balancing sensitivity against alert fatigue across ~45,000 polling stations.

---

## 7. Budget segregation of duties (M54)

Implemented in `crates/budget/src/sod.rs`.

### 7.1 PaymentVoucher approval rule

The `payment_vouchers` table enforces a four-eyes principle: the officer who creates a `PaymentVoucher` (`created_by`) cannot also approve it (`approved_by`). This is enforced at the API layer before the `approved_by` field is written:

```rust
if voucher.created_by == approver_id {
    return Err(SodError::SelfApproval);
}
```

A separate PG CHECK constraint (`CHECK (created_by != approved_by)`) provides a second enforcement layer at the database level.

### 7.2 PPRA procurement compliance

All vouchers above KES 100,000 require a procurement reference (`ppra_ref`) from the Public Procurement and Asset Disposal Authority (PPRA). Vouchers missing `ppra_ref` above this threshold are rejected with `SodError::PpraRefRequired`. The threshold is configurable via the `budget_config` ConfigMap entry.

### 7.3 IFMIS integration

Approved vouchers are exported to IFMIS (Integrated Financial Management Information System) via the Python adapter `py/ifmis/ifmis_client.py`. The adapter uses OAuth2 mTLS with a PKCS#11 client certificate. Export events are logged in `eel_ifmis_export_log` (`voucher_id`, `exported_at`, `ifmis_ref`).

---

## 8. Payroll engine: 250 k staff in 4 h with training gate (M55)

Implemented in `crates/payroll/src/engine.rs`.

### 8.1 Training gate check

Before a payroll run is initiated for a worker, the engine queries training-svc (`GET /v1/training/enrolments?worker_id={id}&status=passed`) to confirm the worker has passed the mandatory M38 poll-worker training module. Workers without a `passed` enrolment are excluded from the run and written to `payroll_runs.excluded_worker_ids` with reason `TRAINING_NOT_COMPLETE`.

### 8.2 Parallelism and throughput

The 250,000-worker target in < 4 hours requires a throughput of > 17 workers/second. The engine processes workers in county-level batches using `tokio::task::spawn_blocking` with a configurable concurrency level (default: 32 parallel workers). Each batch performs:
1. Training gate check (cached for 5 minutes in Valkey).
2. Gross pay computation (daily rate x days worked, from `staff_contracts` and attendance events).
3. Statutory deduction computation: PAYE (KRA tax tables), NHIF/SHIF, NSSF.
4. Net pay record written to `payroll_disbursements`.

### 8.3 Idempotency

Each payroll run is identified by a `run_id` (UUID7). The Valkey idempotency cache (`SETNX run_id:{run_id}:worker:{worker_id}`) prevents a worker from being paid twice in the same run if the engine restarts mid-run.

### 8.4 Statutory deductions

| Deduction | Basis | Implementation |
|---|---|---|
| PAYE | KRA income tax bands (Finance Act, current rates) | payroll/src/tax/paye.rs |
| NHIF/SHIF | NHIF Act (as amended by SHIF transition) | payroll/src/tax/nhif.rs |
| NSSF | NSSF Act 2013 (Tier I + Tier II) | payroll/src/tax/nssf.rs |

Tax rate tables are loaded from ConfigMap at startup; no hardcoded rates in the binary.

---

## 9. M-Pesa B2C batched disbursement (M55)

Implemented in `py/mpesa-b2c/disburse.py`.

### 9.1 Batch construction

Disbursements are grouped into batches of up to 1,000 workers per Safaricom Daraja B2C request. Each batch is assigned a UUID7 `batch_id` used as the `Originator-Conversation-ID` for idempotency. If the batch request fails, it can be retried with the same `batch_id` without risk of double payment, because Daraja rejects duplicate `Originator-Conversation-ID` values.

### 9.2 Result polling

Daraja B2C results are delivered to a registered result URL (`POST /v1/support/payroll/b2c-result`). Each result record is matched to a `payroll_disbursements` row by `mpesa_conversation_id`. Successful results set `disbursed_at`; failed results set `failure_code` and queue a retry.

### 9.3 Retry policy

Failed disbursements are retried with exponential backoff (initial delay 30 s, max 4 retries, max delay 8 min) using Valkey for retry state. After 4 failures, the disbursement is flagged `manual_review_required: true` and escalated to the payroll operations team.

---

## 10. BagIt archive packaging with SHA-256 attestation (M69)

Implemented in `py/bagit/packager.py` and `crates/archive/src/ingest.rs`.

### 10.1 BagIt package structure

Archives are packaged per BagIt v1.0 (RFC 8493, Kunze et al., IETF, 2018):

```
<bag_name>/
  bagit.txt               (version + encoding declaration)
  bag-info.txt            (metadata: Source-Organization, Bagging-Date, Bag-Size, Payload-Oxum)
  manifest-sha256.txt     (sha256 hash + path for every payload file)
  tagmanifest-sha256.txt  (sha256 hash of bag-info.txt + manifest-sha256.txt)
  data/
    <payload files>
```

The `manifest-sha256.txt` is the primary integrity record. Every payload file's SHA-256 is computed during ingest and written to the manifest. On retrieval, the manifest is re-verified before the payload is served.

### 10.2 OAIS information package model

The archive implements the Open Archival Information System (OAIS) reference model (ISO 14721:2012) with three package types:
- **SIP** (Submission Information Package): incoming election records from rts-svc, field-mobile-svc, and other producers.
- **AIP** (Archival Information Package): the BagIt package stored in MinIO with preservation metadata.
- **DIP** (Dissemination Information Package): on-demand extract with provenance manifest for OAG and parliamentary requests.

### 10.3 DataCite metadata

Each archive collection registers a DOI (ISO 26324:2022, Digital Object Identifier System) via the DataCite Metadata Schema v4.4 (DataCite e.V., 2021). Metadata fields: `title`, `creators`, `publisher` (commission), `publicationYear`, `resourceType` (Dataset), `relatedIdentifiers` (links to related collections), `descriptions`. The DOI is recorded in `archive_collections.doi`.

### 10.4 Transparency log attestation

After a BagIt package is written to MinIO, the SHA-256 of the `manifest-sha256.txt` is submitted to transparency-svc for Sigstore Rekor inclusion. The returned `tlog_hash` is stored in `preservation_attestations` (`collection_id`, `tlog_hash`, `attested_at`). This provides an immutable external timestamp proof that the archive existed in its current state at the attestation time.

### 10.5 Destruction certificates

Under the Kenya Data Protection Act 2019 §40 (right to erasure) and the scheduled destruction of time-limited records, the archive produces a signed destruction certificate when a collection is destroyed. The certificate records `collection_id`, `destroyed_at`, `destroyed_by`, `reason`, and a SHA-256 hash of the destroyed manifest. The certificate is retained permanently.

---

## 11. Key algorithms and references

| Algorithm | Implementation path | Reference |
|---|---|---|
| Dual-biometric facility open | crates/special-voter/src/facility.rs | Kenya Elections Act §44A, Elections (General) Regulations 2012 |
| IDP double-cast guard (serialisable transaction) | crates/special-voter/src/idp.rs | Serialisable isolation: Berenson et al., "A Critique of ANSI SQL Isolation Levels", SIGMOD 1995 |
| s.44A dual sign-off PG CHECK | migrations/V0001__special_voter_schema.sql | Kenya Elections Act s.44A |
| Merkle tree (SHA-256) for IAB sample | crates/special-voter/src/iab_sample.rs | RFC 6962 (Laurie et al., IETF, 2013) |
| IAB seed: blake3(root || iab_card) | crates/special-voter/src/iab_sample.rs | O'Connor et al., "BLAKE3", 2020 |
| Z-score outlier detection | crates/intent/src/outlier.rs | Standard normal distribution; threshold 3.0 sigma |
| PAYE tax computation | crates/payroll/src/tax/paye.rs | KRA Finance Act (current year rates from ConfigMap) |
| NSSF Tier I + Tier II | crates/payroll/src/tax/nssf.rs | NSSF Act 2013 (Kenya) |
| Payroll idempotency (Valkey SETNX) | crates/payroll/src/engine.rs | Cache-aside idempotency; UUID7 run_id |
| M-Pesa B2C idempotency (Originator-Conversation-ID) | py/mpesa-b2c/disburse.py | Safaricom Daraja B2C API Guide, developer.safaricom.co.ke, 2023 |
| BagIt packaging | py/bagit/packager.py | RFC 8493 (Kunze et al., IETF, 2018) |
| OAIS information packages | crates/archive/src/ingest.rs | ISO 14721:2012 (OAIS Reference Model) |
| DataCite metadata and DOI | py/bagit/datacite.py | DataCite Metadata Schema v4.4 (DataCite e.V., 2021); ISO 26324:2022 |
| Budget SoD (four-eyes) | crates/budget/src/sod.rs | PPRA procurement rules; standard internal control principle |

---

## 12. Integration points

| Direction | Service | Mechanism | Data |
|---|---|---|---|
| Upstream | bvr-svc | REST cross-check | Special voter station and voted-status lookup (M25 IDP guard) |
| Upstream | rts-svc | Kafka `rts.form34.published` | Archive ingest (M69) |
| Upstream | field-mobile-svc | Kafka `field-mobile.attendance.checked-in` | Payroll trigger and training gate cross-check (M55) |
| Upstream | transparency-svc | REST (tlog submission) | Archive attestation tlog_hash (M69) |
| Upstream | training-svc | REST `GET /v1/training/enrolments` | Payroll training gate check (M55) |
| Downstream | OAG (M59, governance-svc) | REST / file export | Evidence packages from archive (M69) |
| Downstream | IFMIS | HTTP mTLS | Budget voucher export (M54) |
| Downstream | Safaricom Daraja B2C | HTTP | Payroll disbursement (M55) |
| Downstream | KNADS / DataCite | HTTP + DOI registration | Preservation metadata (M69) |

---

## 13. Standards and compliance

| Standard / statute | Coverage |
|---|---|
| BagIt v1.0 (RFC 8493, IETF, 2018) | Archive package format (M69) |
| ISO 14721:2012 (OAIS Reference Model) | SIP/AIP/DIP information package model (M69) |
| ISO 26324:2022 (DOI) | Persistent identifier for archive collections (M69) |
| DataCite Metadata Schema v4.4 (DataCite e.V., 2021) | Archive collection metadata record (M69) |
| Kenya Data Protection Act 2019 §38, §40 | Right to erasure; destruction certificates (M69) |
| Kenya Data Protection Act 2019 §71 | Cross-border transfer controls for diaspora staff payroll (M55) |
| KNADS Act (Kenya National Archives and Documentation Service Act) | 15-year retention obligation (M69) |
| PPRA Procurement Rules (Public Procurement and Asset Disposal Act 2015) | Segregation of duties; voucher approval (M54) |
| Kenya Elections Act §§44A, 53-55 | Special voter identification; spoilt ballot adjudication (M25, M27) |
| NSSF Act 2013 (Kenya) | Payroll statutory deduction (M55) |
| RFC 6962 (Laurie et al., IETF, 2013) | Merkle tree for IAB sample inclusion proofs (M25) |
| RFC 8493 (Kunze et al., IETF, 2018) | BagIt specification (M69) |

---

## 14. Operational excellence

| Capability | Path | Mechanism |
|---|---|---|
| HPA | deploy/primary + deploy/mirror | 2-10 replicas based on CPU and queue depth |
| PDB | deploy/base | minAvailable 1 |
| Payroll idempotency cache | Valkey | SETNX per (run_id, worker_id); TTL 8 h |
| Archive ingest throughput | crates/archive/src/ingest.rs | Multi-part MinIO upload; target > 1 GB/min |
| IFMIS error rate alert | Prometheus | eel_ifmis_error_rate > 1 % fires warning |
| M-Pesa B2C failure alert | Prometheus | eel_mpesa_b2c_failure_rate > 0.1 % fires warning |
| Payroll duration alert | Prometheus | eel_payroll_run_duration_seconds > 3.5 h fires warning |
| Archive lag alert | Prometheus | eel_archive_lag > 10 min fires warning |
| Training gate Valkey cache | Valkey | 5-minute TTL; reduces training-svc call volume during payroll run |
| Degraded mode: payroll deferred | crates/payroll/src/engine.rs | Payroll halted if IFMIS or Daraja B2C unavailable; queued for retry |

Key metrics: `eel_payroll_run_duration_seconds`, `eel_archive_ingest_bytes_total`, `eel_special_voter_events_total`, `eel_budget_voucher_approved_total`.

---

## 15. Validation harnesses

| Harness | Location | What it tests |
|---|---|---|
| Facility FSM property tests | crates/special-voter/tests/facility_fsm.rs | All valid transitions; invalid transitions rejected; dual-biometric gate |
| IDP double-cast serialisability test | crates/special-voter/tests/idp_race.rs | Concurrent identification events at two facilities; only one succeeds |
| s.44A self-approval rejection | crates/special-voter/tests/s44a.rs | CHECK constraint fires when requesting == countersigning officer |
| IAB Merkle inclusion proof tests | crates/special-voter/tests/iab_sample.rs | Inclusion proof validates against root; 5 % sample count correct |
| Z-score outlier detection tests | crates/intent/tests/outlier.rs | Injected high-rejection station detected; z > 3.0 alert fires |
| Budget SoD self-approval test | crates/budget/tests/sod.rs | SelfApproval error returned; PG CHECK fires |
| Payroll throughput load test | loadtest/payroll-250k.js | 250 k workers processed in < 4 h (k6) |
| Payroll idempotency test | crates/payroll/tests/idempotency.rs | Mid-run restart; no double payment |
| Training gate exclusion test | crates/payroll/tests/training_gate.rs | Worker without passed enrolment excluded from run |
| BagIt manifest integrity test | py/bagit/tests/test_packager.py | Re-verify manifest-sha256.txt after round-trip |
| OAIS DIP provenance test | crates/archive/tests/dip.rs | DIP extract includes provenance manifest |
| DataCite metadata schema test | py/bagit/tests/test_datacite.py | Generated XML validates against DataCite Metadata Schema v4.4 XSD |

---

## 16. Data lifecycle, API surface, and events

### 16.1 Data lifecycle

| Table | Retention |
|---|---|
| eel_special_voter_facilities | Election cycle + 15 years (KNADS Act) |
| eel_special_voter_events | Election cycle + 15 years |
| eel_s44a_overrides | Permanent (electoral integrity record) |
| eel_premark_validations | Election cycle + 5 years |
| eel_adjudication_events | Election cycle + 15 years |
| eel_rejection_rate_alerts | Election cycle + 7 years |
| payment_vouchers | 7 years (Finance Act) |
| staff_contracts, payroll_runs | Election cycle + 15 years (KNADS Act) |
| archive_collections, preservation_attestations | Permanent (or per destruction certificate) |

All tables in M25 and M27 are append-only (PG trigger blocks UPDATE/DELETE on event rows). Payroll run records are immutable after `completed_at` is set.

### 16.2 Encryption at rest

`eel_special_voter_events.voter_id_enc` and `eel_idp_reassignments.voter_id_enc` are BYTEA columns storing the voter ID encrypted with a per-row DEK. The schema comment explicitly states: "voter_id encrypted at rest; never stored plaintext." DEKs wrapped under cycle KEK.

### 16.3 API surface (REST, axum 0.8, port 8080)

- `GET /v1/support/special-voters/{facility_id}`: voters at a facility.
- `POST /v1/support/special-voters/events`: record identification event.
- `GET /v1/support/special-voters/{facility_id}/iab-sample`: IAB 5 % sample with Merkle proofs.
- `POST /v1/support/intent/pre-mark`: pre-mark ballot validation.
- `GET /v1/support/intent/spoilt-ballots/{station_id}`: spoilt ballot query.
- `POST /v1/support/payroll/runs`: trigger payroll run.
- `GET /v1/support/payroll/runs/{id}`: run status and excluded worker list.
- `POST /v1/support/payroll/b2c-result`: Daraja B2C result callback.
- `POST /v1/support/budget/vouchers`: create payment voucher.
- `GET /v1/support/archive/collections`: list archive collections.
- `POST /v1/support/archive/ingest`: ingest election records.
- `GET /healthz` + `/readyz` + `/openapi.json`.

### 16.4 Events

**Emitted:**

| Event type | Trigger |
|---|---|
| `support.special.voter.enrolled` | Special voter identification event recorded (M25) |
| `support.intent.spoilt.adjudicated` | Spoilt ballot adjudication completed (M27) |
| `support.budget.voucher.approved` | PaymentVoucher approved (M54) |
| `support.payroll.run.completed` | Payroll run finished (M55) |
| `support.archive.collection.ingested` | Archive collection written and attested (M69) |

**Consumed:**

| Event type | Source | Purpose |
|---|---|---|
| `field-mobile.attendance.checked-in` | field-mobile-svc | Payroll trigger; attendance input for gross pay (M55) |
| `rts.form34.published` | rts-svc | Archive ingest of form 34 records (M69) |
| `bvr.voter.enrolled` | bvr-svc | Special voter cross-check for IDP guard (M25) |
| `transparency.sth.published` | transparency-svc | Archive attestation anchor (M69) |

---

## Summary

Total footprint: 5 Rust crates (archive, budget, intent, payroll, special-voter), 3 Python adapters (bagit, ifmis, mpesa-b2c), 3 Django admin apps (finance, hr, support-admin), 2 SQL migrations (V0001-V0002, 12 tables), Kafka outbox, MinIO object store, Valkey idempotency cache. Core correctness guarantees: dual-biometric facility open for REMAND and BARRACKS categories, serialisable IDP double-cast guard, s.44A four-eyes override with PG CHECK, Merkle-sealed IAB 5 % sample with inclusion proofs, z-score outlier detection for spoilt ballot rejection anomalies, budget SoD at both API and DB layers, payroll idempotency via Valkey SETNX, BagIt v1.0 archive with OAIS model and transparency-log attestation.
