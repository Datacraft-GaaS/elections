# Voter deregistration appeal (M36)

Module M36 (`cvr-deregistration-appeal`) owned by `bvr-svc`. Provides a structured due-process pathway for citizens who believe a CVR removal (M35) was incorrectly proposed or approved: 30-day primary appeal window, 14-day commissioner review, data minimisation at closure, and quarterly gazette of resolved cases.

## Contents

1. What M36 does (vs M35 removal)
2. Data plane (schema)
3. Appeal intake channels
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M36 does (vs M35 removal)

- **M35** = proposes and approves voter removals from the register (deaths, ID corrections, dormancy).
- **M36** = gives the affected citizen or authorised proxy a formal appeal pathway after a removal event is approved; suspends the removal effect during adjudication; records the outcome.

M36 does not re-do M35 classification logic. It consumes `eel.bvr.removal_approved.v1` events and opens a `DeregistrationCase` that the citizen can contest within 30 days. The commissioner has an additional 14 days to issue a final decision. If the appeal succeeds, M36 emits `eel.bvr.reinstatement_approved.v1` back to M35 to restore the voter record.

## 2. Data plane

Three aggregates (per `services/bvr-svc/crates/cvr-appeal/src/models.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **DeregistrationCase** | Root entity: `case_id` (UUID7), `voter_id_hash`, `removal_event_id`, `opened_at`, `primary_deadline` (opened_at + 30 days), `commissioner_deadline` (primary_deadline + 14 days), `status` (`OPEN / UNDER_REVIEW / UPHELD / OVERTURNED / ABANDONED`), `outcome_gazette_id`. | A |
| **AppealRecord** | Each formal submission by the citizen or proxy: `record_id` (UUID7), `case_id`, `submitter_role` (`CITIZEN / PROXY / LEGAL_REP`), `evidence_sha256[]`, `submitted_at`, `officer_assigned`, `officer_decision`, `officer_rationale_hash`. | A |
| **AppealGazetteEntry** | Quarterly gazette line: `entry_id` (UUID7), `case_id`, `outcome`, `gazette_notice_id`, `published_at`. HSM-signed by Commissioner. | A |

All three tables live in `bvr_appeals` schema, migrations under `services/bvr-svc/migrations/`.

## 3. Appeal intake channels

| Channel | Trigger | Auth | SLA |
|---|---|---|---|
| Self-service web portal | Citizen learns of removal via SMS / portal self-check | OIDC (national ID hash + OTP) | Acknowledge in 24 h |
| In-person regional office | Citizen presents at IEBC office | Officer JWT + SpiceDB `COMMISSION_OFFICER` | Officer opens case same day |
| Proxy / legal rep | Proxy with notarised letter | Officer JWT + proxy document upload | Same as in-person |
| System auto-open | `eel.bvr.removal_approved.v1` triggers auto-case for all removals | Internal service SVID | Immediate |

Every auto-opened case enters `OPEN` state. A citizen who does not submit an `AppealRecord` within 30 days causes the case to auto-close as `ABANDONED` via the scheduled `appeal_deadline_sweep` job.

## 4. Pipeline

```
eel.bvr.removal_approved.v1 consumed
      -> open DeregistrationCase (status = OPEN)
      -> SMS + portal notification to voter (M13 distribution)
      -> citizen submits AppealRecord (evidence upload, S3 presigned)
      -> officer reviews evidence (Phoenix LiveView queue)
            -> decision: UPHOLD or OVERTURN
            -> if OVERTURN -> emit eel.bvr.reinstatement_approved.v1
                           -> M35 restores voter; status = OVERTURNED
            -> if UPHOLD   -> status = UNDER_REVIEW (14-day commissioner window)
                  -> commissioner reviews with HSM key
                  -> if commissioner OVERTURN -> reinstatement
                  -> if commissioner UPHOLD   -> status = UPHELD
      -> on deadline expiry -> appeal_deadline_sweep -> ABANDONED
      -> quarterly: AppealGazetteEntry created + HSM-signed
                 -> gazette posted via integrations/gov-printer/gazette-report-poster
                 -> eel.bvr.appeal_gazette_published.v1
      -> data minimisation: raw evidence files purged at 3 months post-closure
         (only SHA-256 hashes retained per KDPA §26)
```

Outbox pattern via `packages/eel-outbox-rs` ensures no event is lost on PG-commit / Kafka-unavailable splits. Circuit breaker via `packages/eel-resilience-rs` guards the SMS gateway and gov-printer adapter.

## 5. API surface

Contract: `packages/openapi-contracts/bvr/cvr-appeal.yaml`. All writes enforce the `Idempotency-Key` header (backed by `eel_idempotency_keys`).

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /v1/appeals/cases` | Commission officer | List open / in-review cases, filterable by status |
| `POST /v1/appeals/cases` | Officer (on behalf of citizen) | Open case manually (in-person channel) |
| `GET /v1/appeals/cases/{case_id}` | Citizen (OIDC) or officer | Retrieve case detail + timeline |
| `POST /v1/appeals/cases/{case_id}/records` | Citizen (OIDC) or officer | Submit AppealRecord with evidence |
| `PATCH /v1/appeals/cases/{case_id}/records/{record_id}` | Commission officer | Set officer decision (UPHOLD / OVERTURN) + rationale |
| `POST /v1/appeals/cases/{case_id}/commissioner-review` | Commissioner + HSM | Final decision; requires HSM signature body |
| `GET /v1/appeals/gazettes` | Public | List quarterly gazette entries |
| `GET /v1/appeals/cases/{case_id}/status` | Public (30/hr) | Citizen case status check; no internal flags |

All writes HSM-signed via response-signing middleware in `packages/eel-api-server-rs`. All reads and writes audited into `eel_bvr_audit_shadow`.

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Officer appeal queue (LiveView) | `services/bvr-svc/ex/cvr_web` | Officer sees new AppealRecords, reviews evidence, sets decision |
| Commissioner review panel (Django) | `services/bvr-svc/django/gazette_admin` | Commissioner HSM-signs final decisions + quarterly gazette |
| Citizen appeal portal (Astro) | `services/bvr-svc/astro/cvr-public-site` | Citizen submits appeal, uploads evidence, tracks status |
| SMS notifications | `integrations/ussd-gateway/adapter` | Status updates at case open, decision, gazette |

Languages: EN + SW on staff surfaces. Citizen portal: EN + SW + top-5 regional languages. WCAG 2.2 AA across all surfaces.

## 7. Integrity + anti-tamper

- **Evidence chain** - each `AppealRecord` stores `evidence_sha256[]`; files are held in S3 under `eel-appeals-evidence/` with object-lock (WORM) for the 90-day adjudication period. After minimisation, only hashes remain.
- **Decision immutability** - officer and commissioner decisions are written once with an HSM signature. Any attempt to overwrite triggers an audit-shadow discrepancy alert.
- **Reinstatement event chain** - `eel.bvr.reinstatement_approved.v1` carries `case_id` + `appeal_record_id` + HSM signature so the M35 re-insertion is traceable back to the appeal.
- **Deadline enforcement** - `appeal_deadline_sweep` is a cron job (scheduled via `packages/eel-scheduler-rs`) with idempotent run semantics; duplicate sweeps are a no-op.
- **Gazette anchoring** - `AppealGazetteEntry` root hash is a leaf in the P05 transparency log, same pattern as M35 snapshots.
- **Minimisation at closure** - evidence S3 objects are deleted (not just dereferenced) 3 months post-closure; a `minimisation_at` timestamp is recorded in the case row and included in quarterly compliance reports.

## 8. Kenya statute alignment

- **Elections Act §10** - right to appear on the register; M36 operationalises the appeal right implied by removal.
- **Kenya Data Protection Act §26 (minimisation)** - only hashes retained post-closure; evidence files purged on schedule.
- **Kenya Data Protection Act §41 (automated processing safeguard)** - auto-opened cases are not auto-decided; human officer and commissioner must act.
- **Kenya Data Protection Act §47 (access to own data)** - citizen can retrieve their case detail via OIDC-authenticated `GET /v1/appeals/cases/{case_id}`.
- **Elections (Technology) Regulations 2017** - HSM signatures on decisions + gazette anchoring satisfies the signed-record requirements.
- **KDPA §40 (right of erasure reconciled with electoral duty to retain)** - evidence hashes are retained as minimum compliance record; raw PII is purged.

## 9. Honest caveats + external prerequisites

- The appeal portal assumes citizens have access to OIDC login via national ID hash + OTP. Citizens without a phone number on record must use the in-person channel.
- Evidence upload is S3-backed via `integrations/s3-compat/`. The S3-compatible bucket (MinIO on auth.lindela.io) must be provisioned and object-lock enabled before go-live.
- The `appeal_deadline_sweep` job must be enabled in the production scheduler config; it is off-by-default to avoid sweeping during staging.
- Proxy authentication (notarised letter) relies on officer judgment, not a digital verification scheme. A future integration with the Notaries Society of Kenya is noted as a gap.
- Gov-printer gazette schema must be agreed with the Government Printer before the first live quarterly publish. The adapter is written to a documented template but the exact schema version is locked at first live batch.

## 10. Test coverage + operational runbooks

Tests under `services/bvr-svc/crates/cvr-appeal/tests/`: property-based via `proptest` on deadline arithmetic, integration via `testcontainers` for PG + Kafka + Debezium round-trip, chaos scenarios under `chaos/scenarios/eel-cvr-appeal-*.yaml`.

Runbooks under `docs/runbooks/`:

- `bvr-svc.md` - section on appeal module deploy + rollback.
- `pgbackrest-pitr.md` - PITR for appeal data (T1-audit, RPO 15 min, RTO 60 min).
- `election-day-mode-toggle.md` - appeal intake stays open during election freeze; only gazette publish is deferred.

Related documents: spec at `docs/plans/modules/cvr-continuous-voter-register.md`; service capability catalogue at `services/bvr-svc/docs/CAPABILITIES.md`.
