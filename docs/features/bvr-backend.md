# BVR backend (M02)

Module M02 (`bvr-backend`) owned by `bvr-svc`. Central authoritative store of the Kenya voter register: ingests enrolment records from BVR device clients, orchestrates ABIS dedup and adjudication, maintains 30M voter records with county partitioning, publishes scoped registers, and provides back-office tools for Returning Officers.

## Contents

1. What M02 does
2. Data plane
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M02 does

M02 is the single source of truth for the voter register. It receives signed enrolment batches from M01 kits, validates IPRS identity, enqueues 1:N dedup requests to M03 (ABIS), adjudicates conflicts, and generates scoped registers at station / ward / constituency / county / national granularity. Every state change is HSM-countersigned and Merkle-anchored in the transparency log (P05).

## 2. Data plane

Per `services/bvr-svc/crates/bvr-domain/src/` and migrations `V1`-`V6`:

| Aggregate | Purpose | Data class |
|---|---|---|
| **Voter** | voter_id (UUID7), 22 biographic fields, status (ACTIVE/DECEASED/DELETED/TRANSFERRED/SUSPENDED), version, election_scope | A |
| **BiometricTemplate** | template_id, voter_id, modality, field-encrypted blob, key_id (per-field KEK via eel-crypto-rs) | A |
| **IDDocumentScan** | scan_id, voter_id, MRZ parsed, encrypted PDF ref (MinIO), SHA-256 hash | A |
| **AuditEvent** | event_id, actor, action, ts_hlc, originator Ed25519 sig, HSM countersig | C |
| **MerkleRoot** | root_hash, seq, signed_by_hsm, escrow_confirmed | C |
| **AdjudicationCase** | case_id, voter_ids, modality, status, investigator_id, decision, reason_code | B |

County partitioning via `V1__create_voters.sql`; audit-shadow triggers via `V2__audit_shadow.sql`; snapshots via `V3__snapshots.sql`.

## 3. Ingest sources

| Source | Adapter | Auth | Failure behaviour |
|---|---|---|---|
| BVR kits (M01) via gRPC | `integrations/bvr-backend/` | SVID mTLS | Circuit breaker; queue locally on kit; server dedup by `eel_idempotency_key` |
| IPRS (National Registration Bureau ID validation) | `integrations/iprs/` | SVID mTLS + IPRS secret | Fail-closed on new enrolments after 24 h down; cache last-known status |
| ABIS match results (M03) | Kafka `eel.abis.match.v1` | SVID mTLS | Circuit breaker; dedup queue accumulates; alert if backlog > 1 h |
| Officer manual corrections | Django back-office | JWT + SpiceDB `COMMISSION_OFFICER` | Dual-control approval |

## 4. Pipeline

Crates: `services/bvr-svc/crates/bvr-api/`, `bvr-domain/`, `bvr-batch/`, `cvr/`.

```
kit POST /bvr/v1/enrolments (SVID mTLS, signed batch)
      -> verify Ed25519 per-record signature + kit-cert binding
      -> verify KitAttestation (TPM quote + firmware digest)
      -> Idempotency-Key dedup (PG-durable eel_idempotency_keys)
      -> IPRS validation (ID, name, DOB); fail-closed if IPRS down > 24 h
      -> store Voter row (versioned append, no in-place delete)
      -> field-encrypt BiometricTemplate with per-field KEK
      -> store IDDocumentScan reference in MinIO
      -> enqueue ABIS 1:N match request (M03) via outbox
      -> ABIS result routes:
             match > threshold -> open AdjudicationCase
             no match          -> accept record, emit eel.bvr.enrolment_accepted.v1
             below quality     -> request re-enrolment from kit
      -> HSM countersign every state change (P06 via integrations/hsm-pkcs11/)
      -> Merkle-append to P05 transparency log via integrations/tlog/
      -> emit eel.bvr.enrolment_accepted.v1 / .rejected.v1 (Kafka eel.bvr.voter.v1)
      -> Debezium CDC picks up outbox -> publishes to Kafka
```

Register generation: streaming sort (deterministic per polling station), bounded-memory, checkpoint-resumable; 30M record PDF completes in <= 30 min. Freeze race at T-60 / T-30 guarded by OPA hard-close (FR-M02-064, TLA+ verified).

## 5. API surface

Contracts: `packages/openapi-contracts/bvr/openapi.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /bvr/v1/enrolments` (gRPC) | Kit (SVID mTLS) | Signed enrolment batch ingest |
| `PATCH /bvr/v1/voters/{id}/status` | RO (JWT + SpiceDB) | Status lifecycle change; biometric-auth gated |
| `POST /bvr/v1/transfers` | RO (JWT + SpiceDB) | Voter transfer; dual-auth |
| `GET /bvr/v1/registers/{scope}` | Bulk (JWT + SpiceDB) | Scoped register generation; dual-control |
| `GET /bvr/v1/voters/{id}` | Accredited (JWT + SpiceDB) | Single voter lookup; field-level redaction |
| `POST /bvr/v1/sar` | Commission officer (JWT + SpiceDB) | ODPC subject access request |
| `POST /bvr/v1/handoff/{token}` | Authenticated user | VPRS handoff token redemption |
| `GET /bvr/v1/snapshots` | Commission admin | List register snapshots |
| `GET /bvr/v1/snapshots/{id}/proof` | Court registrar | Merkle inclusion proof |

Adjudication routes: `services/bvr-svc/crates/bvr-api/src/routes/adjudication.rs`. Merkle routes: `crates/bvr-api/src/routes/merkle.rs`.

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| RO back-office (Django) | `services/bvr-svc/django/bvr-admin/` | Voter status management; adjudication queue; bulk export with DLP |
| ODPC SAR workflow (Django) | `services/bvr-svc/django/bvr-admin/` | Subject access, rectification, erasure, portability |
| Register preview + Merkle proof viewer | REST + Django | Court registrars; dual-control export approval |

Languages: English + Kiswahili. 11-language support in back-office; no truncation at 150% pseudo-locale; ICU plurals.

## 7. Integrity + anti-tamper

- **Per-record Ed25519 signature** verified on every ingest; tampered records emit `eel_tamper` event.
- **HSM countersignature** on every state change (P06); STH pauses if HSM unavailable; countersign queue drains on restore.
- **Merkle-anchored snapshots** via P05 transparency log; Merkle root published on every register snapshot and on configurable batch interval; inclusion proofs available via API.
- **Audit-shadow triggers** (`V2__audit_shadow.sql`) -- every voter row mutation appended to `eel_bvr_audit_shadow`; no in-place delete.
- **OPA hard-close** at register-freeze transaction time; atomicity verified by TLA+ spec and chaos test.
- **Escrow mirror** -- snapshot published to escrow on freeze; `eel_escrow_lag` alert if mirror > 5 min behind primary.
- **Canary reads** (`V5__canary_reads_and_outbox.sql`) -- seeded rows round-trip check; batch rejected on miss.
- **IPRS circuit breaker** via `packages/eel-resilience-rs`; fail-closed on new enrolments after 24 h down.
- **Split-view detection** on transparency log witnesses; consistency check fails closed; alert paged.
- **Voter enumeration defence** -- constant-time response shape; per-requester caps; step-up auth on bulk lookups.

## 8. Kenya statute alignment

- **Elections Act s.6A (public inspection)** -- register generation at every scope; PDF + CSV; Merkle inclusion proof for court registrars.
- **Elections Act (register freeze)** -- OPA hard-close at T-60 and T-30; atomic across all in-flight writes; TLA+ invariant verified.
- **Kenya Data Protection Act 2019 s.26 (minimisation)** -- snapshots store hashed national IDs; raw IDs never leave T1-voter.
- **Kenya Data Protection Act 2019 s.40 (right of erasure)** -- SAR workflow; erasure proof in T1-audit; erasure acknowledged in SAR response within 30 days.
- **Kenya Data Protection Act 2019 s.41 (automated processing)** -- every ABIS match-proposed removal requires officer approval; no fully-automated removals.
- **Elections (Technology) Regulations 2017** -- KEBS-certifier attestation; signed snapshots; Merkle anchoring before each quarterly publish.

## 9. Honest caveats + external prerequisites

- **IPRS SLA** is not contractually defined; circuit-breaker thresholds need agreement with NRB before G3.
- **30M confidential-compute workers** for biometric matching (M03) need infrastructure allocation confirmed.
- **Escrow custodian** identity referenced in spec is unnamed; IEBC decision required before G2.
- **ABIS p95 latency target** (5 s at 30M) depends on M03 infrastructure; measured as M03 integration, not M02 standalone.
- `packages/openapi-contracts/bvr/backend.yaml` referenced in plan does not exist on disk; active contracts are in `packages/openapi-contracts/bvr/openapi.yaml` and `cvr/openapi.yaml`.

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (signature verifier, register-freeze FSM, IPRS circuit-breaker, retention scheduler) | nextest | M02-UNIT- |
| Property (queue never loses record; freeze atomic; HLC monotonic) | proptest | M02-PROP- |
| Contract (M01 ingest; M03 ABIS callback; M07 register-read; VPRS handoff) | Pact | M02-PACT- |
| Integration (IPRS double; ABIS double; HSM SoftHSM; Merkle log testcontainer) | nextest + testcontainers | M02-INT- |
| Formal (register-freeze atomicity TLA+; Merkle append-only kani) | TLC + kani | M02-FORMAL- |
| Load (1,000 records/sec 10 min; 30M generation; bulk export) | k6 + pg bench | M02-LOAD- |
| Chaos (primary kill; IPRS blackhole; HSM kill; Kafka ISR shrink) | Chaos Mesh | M02-CHAOS- |
| Security (Semgrep; ASVS L3; ZAP; STRIDE) | Semgrep + ZAP | M02-SEC- |

Runbooks (to create): `docs/runbooks/bvr-svc-degradation.md`, `docs/runbooks/bvr-svc-register-freeze.md`, `docs/runbooks/pgbackrest-pitr.md`.

Dashboard: `platform/observability/bvr-svc/grafana-ingest.json` (ingest rate, ABIS queue depth, adjudication SLA, escrow lag).

Full spec: `docs/plans/modules/bvr-backend.md`. Service capabilities: `services/bvr-svc/docs/CAPABILITIES.md`.
