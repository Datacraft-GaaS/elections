# Party primaries oversight (M46)

Module M46 (`party-primaries-oversight`) owned by `crms-svc` (crate `crms-api`). Provides IEBC oversight of party primaries: opt-in registration, KIEMS-lite kit support for participating parties, Merkle-anchored result publication, dispute intake routed to the PPDT/PPLC, and observer accreditation linkage to M43. M45 `PoliticalParty.status = ACTIVE` is a hard prerequisite gate.

## Contents

1. What M46 does (vs M45 party registration and M12 candidate management)
2. Data plane (schema)
3. Opt-in and kit-lite modes
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M46 does (vs M45 party registration and M12 candidate management)

- **M45** = party legal existence and status; M46 gates on M45 `status = ACTIVE`.
- **M12** = candidate registration; primary results feed M12 to confirm the nominated candidate.
- **M46** = IEBC's oversight layer for the highest-fraud-risk event in the Kenyan electoral cycle. Parties may opt in to use IEBC's KIEMS-lite kit for biometric voter verification during primaries. Whether or not they use the kit, they must submit results to M46 for publication and oversight.

The distinction between M46 (oversight) and M12 (candidate record) is: M46 publishes a `PrimaryResult` per contest; M12 consumes it to set the `nominated_candidate_id` on the candidate record.

## 2. Data plane

Four aggregates (per `services/crms-svc/crates/crms-domain/src/models/primaries.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **PrimaryOptIn** | Party's decision to use KIEMS-lite kit and/or submit results to IEBC: `opt_in_id` (UUID7), `party_id`, `election_cycle`, `kit_lite_requested` (boolean), `kit_count_requested`, `result_submission_committed` (boolean), `opted_in_at`, `status` (`PENDING / APPROVED / KIT_DISPATCHED / RESULTS_SUBMITTED / CLOSED`). | B |
| **PrimaryResult** | Contest result submitted by party: `result_id` (UUID7), `opt_in_id`, `constituency_code`, `contest_type` (`MP / SENATOR / GOVERNOR / WOMEN_REP / MCA`), `candidate_results[]` (`{name_hash, votes}`), `total_votes_cast`, `submitted_at`, `verified_by_officer`, `published_at`, `merkle_anchor_id`. | A |
| **PrimaryDispute** | Dispute over primary result: `dispute_id` (UUID7), `result_id`, `complainant_id_hash`, `grounds` (`RIGGING / EXCLUSION / KIT_FAILURE / MISCONDUCT`), `filed_at`, `ppdt_ref`, `pplc_ref`, `status` (`FILED / REFERRED / DETERMINATION_ISSUED / CLOSED`). | B |
| **OversightNotice** | IEBC formal notice to party on compliance breach: `notice_id` (UUID7), `party_id`, `notice_type` (`LATE_SUBMISSION / RESULT_DISCREPANCY / KIT_MISUSE`), `issued_at`, `response_deadline`, `party_response_hash`. | B |

## 3. Opt-in and kit-lite modes

| Mode | Description | Kit involved |
|---|---|---|
| Full KIEMS-lite | Party opts in; IEBC dispatches BVR-lite handsets (subset of M06 BVI kit) with pre-loaded party member register; biometric verification at primary polling station | Yes; kit count from M20 inventory |
| Manual with IEBC oversight | Party conducts own primary without IEBC kit; submits signed result forms to M46 within 48 hours of primary date | No |
| Observer-monitored | M43-accredited observers attend; observer reports fed into M46 oversight dashboard | No kit; observer linkage via M43 |

Kit-lite dispatch is coordinated via M20 inventory management (`eel.inventory.kit_dispatched.v1`). Kit returns are tracked via M39 custody events for any ballot material distributed alongside the kit.

## 4. Pipeline

```
party files PrimaryOptIn via portal (M45 ACTIVE gate enforced)
      -> if kit_lite_requested: IEBC reviews capacity; approves if kit stock available (M20 check)
      -> status -> APPROVED
      -> if kit: eel.crms.primaries.kit_required.v1 -> M20 dispatches kit
      -> primary day: kit-lite handsets operate with pre-loaded party member register
      -> party submits PrimaryResult within 48 hours (form upload or API)
      -> IEBC officer verifies result against kit telemetry (if kit-lite) or observer reports
      -> if discrepancy: OversightNotice issued
      -> on officer approval: result published on public primary results portal
            -> Merkle anchor via P05 (merkle_anchor_id stored)
            -> emit eel.crms.primaries.primary_result_published.v1
            -> M12 consumes event to set nominated_candidate_id
      -> dispute filing window: 7 days after publish
            -> PrimaryDispute filed via portal (OIDC complainant)
            -> referred to PPDT (elections disputes) or PPLC (intra-party)
            -> on determination: status -> DETERMINATION_ISSUED
      -> M44 turnout forecasting ingests primary turnout data as historical training signal
      -> opt_in status -> CLOSED on all disputes resolved + kit returned
```

## 5. API surface

Contract: `packages/openapi-contracts/crms/primaries.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/primaries/opt-in` | Party secretary (OIDC) | File PrimaryOptIn |
| `GET /v1/primaries/opt-in/{opt_in_id}` | Party secretary or IEBC officer | Opt-in status + kit allocation |
| `POST /v1/primaries/opt-in/{opt_in_id}/approve` | IEBC officer | Approve + trigger kit allocation |
| `POST /v1/primaries/results` | Party secretary (OIDC) | Submit PrimaryResult |
| `GET /v1/primaries/results` | Public (60/hr) | List published results; no unpublished results visible |
| `PATCH /v1/primaries/results/{result_id}/publish` | IEBC officer | Verify + publish result |
| `POST /v1/primaries/disputes` | Party member (OIDC) | File PrimaryDispute |
| `PATCH /v1/primaries/disputes/{dispute_id}/determination` | IEBC officer | Record PPDT/PPLC determination |
| `POST /v1/primaries/oversight/notices` | IEBC officer | Issue OversightNotice |
| `GET /v1/primaries/oversight/dashboard` | IEBC oversight officer | Aggregate status dashboard |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Party opt-in portal (Astro) | `services/crms-svc/astro/primaries` | Party files opt-in, submits results, files disputes |
| IEBC oversight dashboard (LiveView) | `services/crms-svc/ex/primaries_web` | Officers track opt-ins, results, disputes, notices |
| Public primary results portal (Astro) | `services/crms-svc/astro/primaries` | Public views published primary results |
| Commission admin panel (Django) | `services/crms-svc/django/governance-admin` (shared) | Bulk oversight notice generation; PPDT submission |

Languages: EN + SW on all surfaces. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **Merkle-anchored results** - `PrimaryResult.merkle_anchor_id` records the P05 leaf. A party or petitioner can request the inclusion proof to demonstrate a result was published at a specific time and has not been modified since.
- **Result immutability post-publish** - `PrimaryResult` rows are locked for UPDATE after `published_at` is set. A correction requires an officer-issued replacement result with a new `result_id` and a `supersedes_result_id` reference; both are anchored in P05.
- **Kit telemetry cross-check** - for kit-lite primaries, the kit uploads biometric verification counts (via `eel.crms.primaries.kit_telemetry.v1`). The officer verification step cross-checks submitted `total_votes_cast` against the kit count. A >5% discrepancy triggers an `OversightNotice` automatically.
- **48-hour submission deadline** - the system enforces the 48-hour result submission window. A late submission is flagged and requires officer override with a recorded justification; `OversightNotice(LATE_SUBMISSION)` is auto-issued.
- **Dispute filing window enforcement** - `PrimaryDispute` creation is blocked if `filed_at > result.published_at + 7 days`. The window is configurable per election cycle but cannot be set to <3 days.

## 8. Kenya statute alignment

- **Elections Act §23** - parties must conduct primaries under IEBC supervision; M46 operationalises that supervision.
- **Political Parties Act 2011 §26** - parties must observe democratic principles in nominations; M46's oversight and dispute mechanism enforces accountability.
- **Political Parties Disputes Tribunal Rules 2017** - PPDT jurisdiction over primary disputes; M46's `PrimaryDispute.ppdt_ref` tracks the PPDT reference number.
- **Elections (General) Regulations 2012, Reg 58** - nomination through party primary; M46's published result is the evidentiary base for M12 nomination.

## 9. Honest caveats + external prerequisites

- KIEMS-lite kit availability depends on M20 inventory stock. If the kit pool is exhausted (e.g., multiple parties requesting simultaneous primaries), IEBC must prioritise. M46 exposes `kit_count_available` via the oversight dashboard but does not auto-allocate; officer judgment is required.
- PPDT and PPLC API integrations (`integrations/ppdt-api/`, `integrations/pplc-api/`) are subject to the same caveat as M45: machine-readable APIs are assumed; manual fallback is email-based submission.
- Party-submitted results are self-reported. IEBC's oversight is verification, not independent counting. For kit-lite parties, the verification is stronger (biometric count cross-check); for manual parties, the officer's judgment is the only non-party check.
- M44 turnout forecasting uses M46 historical primary turnout as a training signal. If M46 data quality is poor (e.g., results not submitted), M44 model accuracy for affected constituencies is reduced.

## 10. Test coverage + operational runbooks

Tests under `services/crms-svc/tests/`: opt-in lifecycle integration tests, kit telemetry cross-check arithmetic via `proptest`, dispute filing window enforcement, Merkle anchor round-trip.

Runbooks under `docs/runbooks/`:

- `crms-svc.md` - primaries module deploy + rollback.
- `kit-lite-dispatch.md` - kit allocation + dispatch coordination with M20.
- `late-result-override.md` - officer override procedure for late result submissions.

Related documents: spec at `docs/plans/modules/party-primaries-oversight.md`; M45 at `docs/features/party-registration-management.md`; M12 at `docs/features/`; capability catalogue at `services/crms-svc/docs/CAPABILITIES.md`.
