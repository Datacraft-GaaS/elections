# Voter list public inspection (M64)

Module M64 (`voter-list-public-inspection`) owned by `portal-svc`. Publishes verifiable roll snapshots for the statutory public inspection window (T-60 days before the election), accepts objections and correction claims from citizens, records IEBC determinations, and packages auto-referral packets for cases escalated to the courts or the Registrar of Voters.

## Contents

1. What Voter List Public Inspection does (vs M35 CVR)
2. Data plane (schema)
3. T-60 idempotent scheduler
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What Voter List Public Inspection does (vs M35 CVR)

- **M35** (Continuous Voter Registration) = ongoing maintenance of the voter register: death removal, ID corrections, dormancy flags. It produces the definitive register.
- **M64** = statutory public window layer: takes a point-in-time `RegisterSnapshot` from M35, publishes it as a public-inspection roll, accepts citizen challenges, and records determinations.

M64 does not modify the voter register directly. A successful `Determination` in M64 emits an event that M35 processes as a `CORRECTION` input, which then goes through M35's officer triage and commissioner approval before the register changes.

## 2. Data plane

Five aggregates (per `services/portal-svc/crates/voter-inspection/src/models.rs` and the M64 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **RollSnapshot** | Point-in-time voter roll published for inspection: `snapshot_id`, `election_cycle_id`, `source_snapshot_id` (from M35 `RegisterSnapshot`), `published_at`, `inspection_window_start`, `inspection_window_end`, `total_registered`, `root_hash` (SHA-256), commissioner HSM signature. | A |
| **Objection** | Citizen challenge to a voter's inclusion: `objection_id`, `snapshot_id`, `objector_national_id_hash`, `challenged_voter_national_id_hash`, `grounds` (DUPLICATE / DECEASED / NON_CITIZEN / INELIGIBLE), `evidence_urls` (MinIO), `submitted_at`. | B |
| **CorrectionClaim** | Citizen claim that their own record is wrong: `claim_id`, `snapshot_id`, `claimant_national_id_hash`, `field` (NAME / DOB / ADDRESS / POLLING_STATION), `current_value_md5`, `proposed_value`, `evidence_urls` (MinIO), `submitted_at`. | B |
| **Determination** | IEBC officer ruling on an Objection or CorrectionClaim: `determination_id`, `objection_id` or `claim_id`, `outcome` (UPHELD / DISMISSED / REFERRED_TO_COURT / REFERRED_TO_REGISTRAR), `reasoning_md`, `determined_by_officer`, `determined_at`. | A |
| **AutoReferralPacket** | Evidence package assembled for court or registrar referral: `packet_id`, `determination_id`, `referral_type` (COURT / REGISTRAR), `content_urls` (MinIO), `referral_ref`, `submitted_at`. | B |

## 3. T-60 idempotent scheduler

The inspection window must open exactly T-60 days before election day. A cron job triggers the `PublishRollSnapshot` command 60 days before the gazetted election date. The job is idempotent: if a `RollSnapshot` already exists for `election_cycle_id`, it returns early without creating a duplicate. Idempotency is enforced by a Postgres unique constraint on `(election_cycle_id, source_snapshot_id)`.

The election date is read from the `election_calendar` table owned by M52 (Election Calendar and Gazette). If M52 has not set a confirmed election date, the T-60 scheduler will not fire; this is logged as a warning, not an error.

## 4. Pipeline

Files on disk under `services/portal-svc/crates/voter-inspection/src`: `lib.rs`, `models.rs`, `snapshot.rs`, `objection.rs`, `correction.rs`, `determination.rs`, `referral.rs`, `routes/`.

```
T-60 cron fires (from election_calendar, M52)
      -> fetch latest RegisterSnapshot from M35
      -> create RollSnapshot; commissioner signs with HSM
      -> emit eel.portal.roll_snapshot_published.v1
      -> public inspection portal opens (Astro public site)
      -> citizens submit Objections and CorrectionClaims
            -> evidence uploaded to MinIO; URLs stored
            -> emit eel.portal.objection_submitted.v1
               or eel.portal.correction_claim_submitted.v1
      -> IEBC officer reviews each submission
            -> creates Determination (UPHELD / DISMISSED / REFERRED)
            -> emit eel.portal.determination_made.v1
            -> if UPHELD or DISMISSED:
                  -> M35 receives UPHELD event; M35 officer triage begins
            -> if REFERRED_TO_COURT or REFERRED_TO_REGISTRAR:
                  -> create AutoReferralPacket
                  -> package assembled and submitted to court / registrar
                  -> emit eel.portal.referral_packet_submitted.v1
      -> inspection window closes at inspection_window_end
      -> emit eel.portal.inspection_window_closed.v1
```

## 5. API surface

Contract: `packages/openapi-contracts/portal/voter-inspection.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /v1/voter-inspection/snapshots/{cycle_id}` | Public (rate-limited 60/hr) | Current roll snapshot metadata + root hash |
| `GET /v1/voter-inspection/snapshots/{id}/check/{national_id_hash}` | Public (rate-limited 30/hr) | Check if a hashed national ID is on the roll |
| `POST /v1/voter-inspection/objections` | Citizen (public JWT, rate-limited 5/day) | Submit objection |
| `POST /v1/voter-inspection/correction-claims` | Citizen (public JWT, rate-limited 5/day) | Submit correction claim |
| `GET /v1/voter-inspection/objections` | IEBC officer (JWT + SpiceDB `INSPECTION_OFFICER`) | List objections for triage |
| `GET /v1/voter-inspection/correction-claims` | IEBC officer | List correction claims for triage |
| `POST /v1/voter-inspection/determinations` | IEBC officer | Record determination |
| `GET /v1/voter-inspection/determinations/{id}/referral` | Legal officer | Auto-referral packet status |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Public inspection portal (Astro) | `services/portal-svc/astro/voter-inspection-public` | Citizens search the roll, submit objections and corrections |
| IEBC officer triage console (Django) | `services/portal-svc/django/inspection_admin` | Review submissions, record determinations |
| Referral management (Django) | `services/portal-svc/django/inspection_admin` | Assemble and track auto-referral packets |

Languages: EN + SW + Kikuyu + Luo + Kamba (public portal). WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **HSM-signed snapshot**: commissioner HSM signature over `SHA-256(snapshot_id || root_hash || total_registered || published_at)`. The public portal displays the signature so any citizen can verify it.
- **Root hash verification**: the public can verify their national ID hash is (or is not) in the roll by requesting the Merkle inclusion proof (same mechanism as M35's P05 anchoring).
- **Idempotent T-60 scheduler**: Postgres unique constraint prevents duplicate snapshots per election cycle; the scheduler logs a warning and returns the existing snapshot ID if called more than once.
- **Objection rate limiting**: citizens are limited to 5 objections per day per IP and national ID hash to prevent abuse.
- **Evidence immutability**: MinIO objects referenced by `Objection.evidence_urls` are stored with a retention lock; they cannot be deleted during the inspection window.
- **Audit shadow**: all determination and referral events logged to `eel_portal_audit_shadow`.

## 8. Kenya statute alignment

- **Elections Act §6A (public inspection of register)**: the inspection window (T-60 to T-30 is the typical statutory window) and the `RollSnapshot` publish are the mechanism for satisfying this requirement.
- **Elections Act §7 (objections and corrections)**: `Objection` and `CorrectionClaim` aggregates map to the statutory right to challenge and correct the register.
- **Elections Act §8 (determination of claims)**: `Determination` records the officer ruling required by §8.
- **Kenya Data Protection Act §26 (minimisation)**: citizens search using their own hashed national ID; raw national IDs are not displayed, transmitted, or stored in M64 (they are hashed before submission).

## 9. Honest caveats + external prerequisites

- The T-60 scheduler requires M52 to have gazetted the election date in the `election_calendar` table before T-60. If IEBC gazettes the election date late, the inspection window may open late, potentially breaching the Elections Act §6A requirement.
- Evidence upload (MinIO) for objections assumes internet access by citizens submitting evidence; citizens without internet access must visit a constituency office to submit physical evidence, which the officer uploads on their behalf.
- The auto-referral packet is a digital evidence package; actual court filing may require physical delivery in addition to the digital submission. M64 records the digital submission; physical delivery is a manual operational step.
- Objection and correction claim volumes may exceed expectations in high-turnout constituencies; the portal is load-tested at 10,000 concurrent users but actual peaks are hard to predict.

## 10. Test coverage + operational runbooks

Coverage under `services/portal-svc/crates/voter-inspection/tests/`: T-60 idempotency (integration: two cron fires produce one snapshot), HSM signature verification (integration), objection rate limiting (load test), Merkle inclusion proof (round-trip with M35 mock), determination emits M35 correction event (integration).

Runbooks under `docs/runbooks/`:

- `portal-svc.md` (section: voter-list-public-inspection) -- deploy, rollback, inspection window management.
- `inspection-window-late-open.md` -- steps when T-60 has passed without a gazetted election date.
- `objection-volume-spike.md` -- auto-scaling and rate-limit adjustment during high-volume periods.

Related documents: full spec at `docs/plans/modules/voter-list-public-inspection.md`; service capability catalogue at `services/portal-svc/docs/CAPABILITIES.md`; M35 CVR at `docs/features/continuous-voter-registration.md`; M52 at `docs/features/election-calendar-gazette.md`.

### Inspection window timeline

```
T-60: RollSnapshot published; public portal opens; objections/corrections accepted
T-45: Midpoint review; IEBC officer triage queue status report to commissioner
T-30: Inspection window closes; no new objections or corrections accepted
T-30 to T-14: IEBC officers make Determinations on all pending submissions
T-14: AutoReferralPackets submitted to court/registrar for REFERRED cases
T-7: All determinations communicated to objectors via SMS
```

These dates are relative to election day and are derived from the Elections Act §6A + §8 statutory windows. The T-30 window close is enforced by a Postgres constraint: `submitted_at <= (SELECT inspection_window_end FROM roll_snapshots WHERE id = $1)`.

### Objection grounds taxonomy

| Ground | Definition | Evidence required |
|---|---|---|
| DUPLICATE | Voter appears twice in the register | Second entry national ID hash |
| DECEASED | Voter is deceased | CRS death notification reference or certificate |
| NON_CITIZEN | Voter is not a Kenyan citizen | Supporting documentation |
| INELIGIBLE | Voter is otherwise ineligible (e.g., under 18) | Documentary proof |

Evidence is uploaded to MinIO at submission time; the officer reviews during triage. Frivolous objections (no evidence uploaded) are fast-tracked to DISMISSED with a one-sentence reasoning note.
