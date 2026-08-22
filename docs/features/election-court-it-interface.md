# Election court IT interface (M40)

Module M40 (`election-court-it-interface`) owned by `governance-svc` (crate `court-interface`). Assembles and delivers a 15-item evidentiary bundle to the Supreme Court or High Court within 24 hours of election close; grants petitioner portal access to disclosed materials; handles supplementary evidence requests; manages technical witness scheduling. Addresses the Raila Odinga v IEBC [2017] eKLR para 272 finding that IEBC lacked a structured IT evidence disclosure system.

## Contents

1. What M40 does (vs M41 post-election review)
2. Data plane (schema)
3. Evidence bundle composition
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M40 does (vs M41 post-election review)

- **M41** = post-election lessons-learned and long-term improvement tracking; no legal force.
- **M40** = formal court-facing disclosure: signed evidence bundles, petitioner portal, supplementary requests under court order, and technical witness roster. It operates under judicial timelines, not IEBC internal timelines.

M40 is activated automatically when `eel.declaration.result_declared.v1` is received for a contest with petition potential (all national + county positions). A `EvidenceBundle` is assembled within 24 hours. Petitioners who file in court can apply for portal access; the eCourt integration notifies M40 of accepted petitions automatically.

## 2. Data plane

Five aggregates (per `services/governance-svc/crates/court-interface/src/models.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **EvidenceBundle** | Root record: `bundle_id` (UUID7), `contest_id`, `election_cycle`, `assembled_at`, `item_count` (target 15), `status` (`ASSEMBLING / SEALED / DELIVERED / SUPPLEMENTED`), `bundle_sha256`, `hsm_signature`, `court_ref`. | A |
| **BundleItem** | One item in the bundle: `item_id` (UUID7), `bundle_id`, `item_type` (see below), `source_system`, `source_ref`, `content_sha256`, `s3_key`, `included_at`. | A |
| **PetitionerAccess** | Portal access grant: `access_id` (UUID7), `bundle_id`, `petitioner_id_hash`, `law_firm_ref`, `granted_by` (court order ref), `granted_at`, `expires_at`, `access_scope[]` (subset of item types). | A |
| **SupplementaryRequest** | Court-ordered additional disclosure: `request_id` (UUID7), `bundle_id`, `court_order_ref`, `requested_items[]`, `deadline`, `fulfilled_at`, `fulfilled_sha256`. | A |
| **TechnicalWitness** | IEBC technical expert scheduled to testify: `witness_id` (UUID7), `bundle_id`, `officer_id`, `expertise_tag`, `hearing_date`, `testimony_topic`, `confirmed`. | B |

## 3. Evidence bundle composition

The 15 standard bundle items (per IEBC Strategy §XII.4):

| # | Item type | Source |
|---|---|---|
| 1 | `VOTER_REGISTER_SNAPSHOT` | M35 `RegisterSnapshot` for the contest ward |
| 2 | `BVR_AUDIT_LOG` | M02 biometric verification audit log (anonymised) |
| 3 | `BALLOT_CHAIN_PROOF` | M39 Merkle inclusion proofs for all lots in contest |
| 4 | `FORM_34A_SCANS` | M10 rts-backend captured forms (all stations) |
| 5 | `FORM_34B_TALLIES` | M10 constituency tally forms |
| 6 | `RESULT_TALLY_STREAM` | M10 tally event log (Kafka compacted snapshot) |
| 7 | `ABIS_VERIFICATION_SUMMARY` | M03 ABIS match/no-match counts per station |
| 8 | `KIT_TELEMETRY_LOG` | M08 BVI device telemetry (uptime, error codes) |
| 9 | `HSM_SIGNATURE_CHAIN` | All HSM signatures on results + certificates |
| 10 | `DECLARATION_CERTIFICATE` | M51 `CertificateOfElection` (HSM-countersigned) |
| 11 | `STATION_RECONCILIATION` | M39 `StationReconciliation` exports for all stations |
| 12 | `WORKER_CERT_MANIFEST` | M38 `WorkerCertificate` list for contest stations |
| 13 | `OBSERVER_ACCREDITATION` | M43 accredited observers list for contest |
| 14 | `INCIDENT_LOG` | M16 ECC incident log for contest area |
| 15 | `TRANSPARENCY_LOG_ROOTS` | P05 Merkle tree roots at declaration time |

## 4. Pipeline

```
eel.declaration.result_declared.v1 consumed
      -> EvidenceBundle created (status = ASSEMBLING)
      -> 15-item assembly workers spawn (parallel per item type)
            -> each worker queries source system via internal gRPC
            -> content written to S3 (eel-court-evidence/); content_sha256 stored
            -> BundleItem row inserted
      -> on all 15 items present (or assembly_deadline T+24h):
            -> bundle_sha256 = SHA-256(sorted item content_sha256 list)
            -> Commissioner HSM signature applied
            -> status -> SEALED
            -> emit eel.governance.court_bundle_sealed.v1
      -> eCourt API push: bundle metadata + secure download link
            -> court_ref stored in EvidenceBundle
      -> on eCourt petition accepted (webhook):
            -> PetitionerAccess created with access_scope per court order
            -> petitioner portal access enabled
      -> on SupplementaryRequest (court order webhook):
            -> additional items assembled per deadline
            -> status -> SUPPLEMENTED; fulfilled_sha256 logged
      -> TechnicalWitness entries managed by Commission legal team
```

## 5. API surface

Contract: `packages/openapi-contracts/governance/governance-court.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /v1/court/bundles` | Commission legal (JWT) | List bundles by status / contest |
| `GET /v1/court/bundles/{bundle_id}` | Commission legal or petitioner (scoped JWT) | Bundle detail + item list |
| `GET /v1/court/bundles/{bundle_id}/items/{item_id}` | Petitioner (scoped JWT) | Download bundle item (if in access_scope) |
| `GET /v1/court/bundles/{bundle_id}/proof` | Court registrar | Merkle inclusion proof for bundle_sha256 |
| `POST /v1/court/bundles/{bundle_id}/access` | Commission legal | Grant PetitionerAccess (on court order) |
| `POST /v1/court/bundles/{bundle_id}/supplementary` | Commission legal | Log SupplementaryRequest + trigger assembly |
| `GET /v1/court/witnesses` | Commission legal | List TechnicalWitness entries |
| `POST /v1/court/witnesses` | Commission legal | Add TechnicalWitness |
| `PATCH /v1/court/witnesses/{witness_id}` | Commission legal | Update hearing date + confirmation |

All responses are TLS 1.3 + IEBC public-key signed. Petitioner downloads are single-use pre-signed S3 URLs (15-minute TTL).

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Bundle assembly console (LiveView) | `services/governance-svc/ex/court_web` | Commission legal monitors assembly progress; seals bundle |
| Petitioner portal (Astro) | `services/governance-svc/astro/court-portal` | Petitioners browse and download disclosed items |
| Commissioner HSM panel (Django) | `services/governance-svc/django/governance-admin` | HSM-sign bundle; authorise supplementary releases |

Languages: EN on petitioner and legal surfaces. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **Immutable assembly** - `BundleItem` rows are insert-only; content_sha256 is computed and stored before the item is written to the DB. No UPDATE permitted.
- **Bundle SHA-256** - `EvidenceBundle.bundle_sha256` is the SHA-256 of the sorted list of item content hashes. Any post-seal mutation is detectable by recomputing the hash.
- **HSM countersignature** - the sealed bundle carries a Commissioner HSM signature verifiable against the public key in the P05 transparency log. A court cryptographer can verify the signature independently.
- **P05 Merkle root anchor** - item 15 in the bundle (`TRANSPARENCY_LOG_ROOTS`) records the P05 Merkle root at declaration time. This binds all earlier M35/M39/M51 Merkle anchors to the same temporal reference.
- **Single-use petitioner downloads** - S3 pre-signed URLs expire in 15 minutes and are single-use (via S3 event notification + download-token revocation). Re-download requires a new request logged in `PetitionerAccess`.
- **Supplementary audit** - every `SupplementaryRequest` records the court order reference; a fulfillment without a court order reference is rejected.

## 8. Kenya statute alignment

- **Elections Act §75** - High Court jurisdiction for election petitions; M40 operationalises the evidence disclosure duty.
- **Supreme Court Act §20** - Supreme Court petition procedure; M40 supports the 14-day petition window with 24-hour bundle assembly.
- **Evidence Act Cap 80 §78A** - electronic evidence admissibility; M40 produces Merkle-anchored, HSM-signed artefacts that satisfy the authenticity requirement.
- **Elections (Technology) Regulations 2017 Reg 17** - Commission shall preserve electronic records for court inspection; M40 is the implementation.
- **Raila Odinga v IEBC [2017] eKLR para 272** - Supreme Court finding that IEBC lacked structured IT disclosure; M40 directly addresses this precedent.

## 9. Honest caveats + external prerequisites

- eCourt integration (`integrations/ecourt/`) requires the Judiciary's eCourt system to expose a webhook API for petition acceptance notifications. The adapter is written to a documented schema; the live endpoint is an external dependency pending Judiciary IT provisioning.
- 24-hour assembly relies on all 15 source systems being reachable and responsive. If a source system (e.g., M03 ABIS) is unavailable, the corresponding `BundleItem` is marked `UNAVAILABLE` and the bundle is sealed with the item marked as absent. The Commission must notify the court of the gap.
- Petitioner access provisioning requires a Commission legal officer to act on a court order. There is no self-service petitioner registration; the court order is the gate.
- Technical witness scheduling (the `TechnicalWitness` aggregate) is managed manually by the Commission legal team. There is no integration with court scheduling systems.

## 10. Test coverage + operational runbooks

Tests under `services/governance-svc/crates/court-interface/tests/`: integration via `testcontainers` for PG + S3 + gRPC mocks for all 15 source systems, chaos scenarios for partial assembly (missing items), signature verification test vectors.

Runbooks under `docs/runbooks/`:

- `governance-svc.md` - deploy, rollback, HSM signing procedure.
- `court-bundle-gap.md` - protocol when a source system is unavailable during assembly.
- `petitioner-access-revocation.md` - revoking access on court order withdrawal.

Related documents: spec at `docs/plans/modules/election-court-it-interface.md`; M39 at `docs/features/ballot-printing-chain.md`; M51 at `docs/features/result-declaration-certification.md`.

---

*Module documentation version: 1.0. Last updated: 2026-08-22. Review before each election cycle to verify crate paths, OpenAPI contract versions, integration endpoint URLs, and statutory rule offsets remain current.*
