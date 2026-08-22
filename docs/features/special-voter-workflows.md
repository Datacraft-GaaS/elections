# Special voter workflows (M25)

Module M25 (`special-voter-workflows`) owned by `support-admin-svc`. Manages five categories of voters with non-standard polling circumstances: remand and convicted prisoners (REMAND/BARRACKS dual-biometric unlock), registered refugees (REFUGEE_VOTING_ENABLED flag with UNHCR IDP roll cross-reference), internally displaced persons (double-cast guard), voters with disabilities and the elderly needing assisted voting (s.44A four-eyes), and hospital patients. IAB audits a 5% Merkle-seeded random sample of assisted votes.

## Contents

1. What special voter workflows does (vs related modules)
2. Data plane (schema)
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What special voter workflows does (vs related modules)

- **M01/M02 BVR** = base enrolment; M25 reads voter records and applies category flags but does not re-enrol.
- **M25** = special-circumstance overrides at polling time: unlocks non-standard biometric paths, enforces category-specific guards (double-cast, four-eyes, dual-biometric), and routes IAB audit samples.
- **M31 Mobile poll clerk** = the DPO companion app that presents M25 workflow prompts to the presiding officer at the station; M25 supplies the workflow definitions, M31 renders them.

M25 emits workflow events but does not certify results. Every special-category voter vote is counted by M09 the same as any other vote; M25 only governs the access and assistance workflow.

## 2. Data plane

Six aggregates under `services/support-admin-svc/crates/special-workflows-api/src/models.rs`:

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **SpecialVoterCategory** | Category assignment: voter_id_hash, category (REMAND / BARRACKS / REFUGEE / IDP / DISABILITY / HOSPITAL), category_ref, assigned_by, valid_until. | B |
| **BiometricUnlock** | Dual-biometric override for REMAND/BARRACKS: voter_id_hash, officer1_sig, officer2_sig, station_id, ts_hlc. | A |
| **RefugeeVoteCheck** | UNHCR IDP roll cross-reference result: voter_id_hash, unhcr_ref, cross_ref_status (FOUND/NOT_FOUND/PENDING), ts. | B |
| **DoubleCastGuard** | IDP double-cast detection: voter_id_hash, home_station_id, displaced_station_id, guard_status (CLEAR/FLAGGED). | A |
| **AssistedVoteRecord** | Four-eyes assisted vote: voter_id_hash, assistant_id, officer_sig, reason_code, ts_hlc, audit_sample_flag. | A |
| **AuditSample** | IAB 5% random sample: audit_id, sampled_from_merkle_root, records (array of AssistedVoteRecord refs), status. | A |

## 3. Ingest sources

| Source | Cadence | Format | Auth | Failure semantics |
|---|---|---|---|---|
| Prison Service roll (REMAND/BARRACKS) | Pre-election | Signed CSV | SVID mTLS | Block if signature fails; manual officer override logged |
| UNHCR IDP roll (REFUGEE) | Pre-election | Signed CSV | SVID mTLS | REFUGEE_VOTING_ENABLED flag off until roll received |
| NCPD disability register | Pre-election | Signed CSV | SVID mTLS | Register unavailable: category remains UNCONFIRMED; officer presents discretionary path |
| IDP displacement notifications | Real-time | REST | JWT (NCPD officer) | Queued; double-cast guard updated on receipt |
| M31 poll clerk workflow events | Polling day | Kafka event | Device cert | At-least-once; idempotent by voter_id_hash + station_id + date |
| Hospital coordinator | Pre-election | Django form | JWT + SpiceDB HOSPITAL_COORD | Hospital station assignment; patient roll slice |

## 4. Pipeline

```
REMAND / BARRACKS path:
  officer opens M31 companion app -> BiometricUnlock workflow
  -> officer1 presents biometric + PIN
  -> officer2 presents biometric + PIN (different officer required)
  -> both biometrics verified against station BVR slice
  -> BiometricUnlock(officer1_sig, officer2_sig) persisted
  -> eel.special.biometric_unlock.v1 emitted

REFUGEE path:
  REFUGEE_VOTING_ENABLED checked (flag from UNHCR MoU confirmation)
  if disabled: voter rejected with advisory message to station coordinator
  if enabled:
    voter presents; voter_id cross-referenced with UNHCR IDP roll
    RefugeeVoteCheck created
    if NOT_FOUND: advisory; officer discretionary path logged

IDP path:
  voter presents at displaced station
  DoubleCastGuard checked against home_station_id
  if FLAGGED: vote blocked; incident logged; officer escalation
  if CLEAR: vote proceeds; DoubleCastGuard updated to DISPLACED_VOTED

DISABILITY / ELDERLY assisted vote (s.44A):
  officer activates four-eyes mode in M31
  -> voter selects or names assistant
  -> officer witnesses; officer_sig captured
  -> AssistedVoteRecord persisted with audit_sample_flag
  -> every 10 min: AssistedVoteRecord hashes included in Merkle root
  -> IAB audit sample: Merkle root seeded random draw of 5%
       AuditSample record created; IAB notified via dashboard

HOSPITAL path:
  hospital station configured pre-election; patient roll slice provisioned
  polling team arrives; biometric match against patient roll slice
  standard flow with AssistedVoteRecord for assisted patients
```

## 5. API surface

Contract: `packages/openapi-contracts/support-admin/special-workflows.yaml`. All write endpoints enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /special/v1/categories` | IEBC officer (JWT + SpiceDB) | Assign special category to voter |
| `POST /special/v1/unlocks` | M31 device cert | Submit dual-biometric unlock event |
| `POST /special/v1/refugee-check` | M31 device cert | Submit UNHCR cross-reference result |
| `GET /special/v1/double-cast/{voter_id_hash}` | M31 device cert | Check IDP double-cast guard |
| `POST /special/v1/assisted` | M31 device cert | Record assisted vote (four-eyes) |
| `GET /special/v1/audit-samples` | IAB officer (JWT + SpiceDB) | IAB 5% audit sample queue |
| `PATCH /special/v1/audit-samples/{id}` | IAB officer | Record audit outcome |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| M31 companion app (Flutter) | `services/field-mobile-svc/flutter/poll-clerk/` | Renders special workflow prompts from M25 definitions |
| Special workflows admin (Django) | `services/support-admin-svc/django/special-admin/` | Pre-election category assignment; REFUGEE flag control; hospital station setup |
| IAB audit dashboard (Phoenix LiveView) | `services/support-admin-svc/ex/special_audit_web/` | IAB views 5% sample queue; records audit outcomes |
| Category reporting (Astro, CDN-cached) | `services/support-admin-svc/astro/special-public/` | Aggregated counts by category (no individual voter data) |

WCAG 2.2 AA on all surfaces. M31 offline mode: all special-workflow checks must function without network connectivity; results queued and synced on reconnect.

## 7. Integrity + anti-tamper

- **Dual-biometric unlock** -- REMAND/BARRACKS unlock requires two distinct officer biometrics verified against station BVR slice; a single-officer attempt is rejected at the API layer before any persistence.
- **IDP double-cast guard** -- the guard record is written before the vote token is issued; a network failure between guard-check and vote issuance causes a safe failure (no token issued; officer re-presents online).
- **IAB Merkle-seeded 5% sample** -- the Merkle root over assisted vote records is used as the seed for random sample selection; any manipulation of the assisted vote records would change the Merkle root and therefore the sample, making post-hoc manipulation detectable.
- **REFUGEE_VOTING_ENABLED flag** -- a bilateral configuration flag that defaults to false; requires both UNHCR roll receipt and IEBC Commissioner approval to set true; audit-logged on every state change.
- **Assisted vote four-eyes** -- officer signature required for every AssistedVoteRecord; no assisted vote is accepted without an officer_sig; unsigned records are rejected by the API and logged as anomalies.
- **Category assignment audit** -- every SpecialVoterCategory assignment and change is append-only and linked to the assigning officer; no officer can retroactively remove a category assignment.

## 8. Kenya statute alignment

- **Elections Act s.44A (assisted voting)** -- four-eyes AssistedVoteRecord with officer witness satisfies the statutory obligation; IAB 5% audit is the oversight mechanism.
- **Elections Act s.52 (prisoners)** -- REMAND/BARRACKS dual-biometric unlock implements the statutory access provision; Prison Service roll is the authoritative list.
- **Kenya Refugees Act 2021** -- REFUGEE_VOTING_ENABLED flag and UNHCR roll cross-reference implement the statutory conditional right; the flag cannot be set without the UNHCR roll being received.
- **DPA 2019 s.26 (minimisation)** -- voter_id_hash (not raw ID) stored in all special workflow records; category_ref field stores only the statutory reference (e.g., prison number, UNHCR registration number) in encrypted form.
- **NCPD Act 2003** -- disability category sourced from NCPD register; M25 does not independently classify disability.

## 9. Honest caveats + external prerequisites

- Prison Service roll MoU is a prerequisite for the REMAND/BARRACKS path; without the signed roll, the dual-biometric unlock path defaults to officer manual entry logged as UNCONFIRMED.
- UNHCR IDP roll data quality varies; NOT_FOUND results do not necessarily mean the voter is ineligible; the officer discretionary path is the fallback but introduces human judgment risk.
- The IDP double-cast guard depends on the voter's home_station_id being correctly recorded in the BVR; incorrect home station assignments are a known data quality risk.
- Hospital station polling team logistics are operationally complex; M25 provides the software workflow but the team deployment and patient consent process are IEBC field operations responsibilities.
- NCPD disability register completeness is unknown; voters with disabilities not on the register must be handled via the officer discretionary path, which reduces auditability.

## 10. Test coverage + operational runbooks

Coverage under `services/support-admin-svc/crates/special-workflows-api/tests/`: unit tests for dual-biometric unlock validator, double-cast guard, REFUGEE flag check, AssistedVoteRecord four-eyes enforcer, Merkle-seeded sample generator; property-based tests asserting double-cast guard cannot be bypassed for the same voter_id_hash + polling day combination; contract tests (Pact) for M31 workflow prompts and IAB audit feed; integration tests with SoftHSM dual-signer, UNHCR stub, Prison Service stub, NCPD stub, Isar offline mode; e2e scenario covering all five category paths including IDP double-cast block and IAB audit sample generation; chaos test where network is unavailable during double-cast guard check (verifies safe failure).

Runbooks under `docs/runbooks/`:

- `special-workflows-degradation.md` (create): Prison Service roll unavailable; UNHCR roll stale; double-cast guard network failure; IAB audit sample generation failure.
- `iap-audit-sampling.md` (create): step-by-step for IAB 5% sample review and outcome recording.

Dashboards: `platform/observability/support-admin-svc/grafana-special.json` (unlock event rate by category, double-cast flags, assisted vote count, IAB audit completion rate). SLO: 99.9% workflow event write success; double-cast guard response <500 ms; IAB sample generated within 30 min of polling close.

Related: `docs/plans/modules/special-voter-workflows.md`; `services/support-admin-svc/docs/CAPABILITIES.md`; edge-case matrix EC-SPV-* in `docs/plans/08-edge-case-matrix.md`.
