# By-election operations (M50)

Module M50 (`by-election-operations`) owned by `governance-svc` (crate `by-election`). Full lifecycle management of by-elections triggered by recall (M48), court-ordered vacancy, or death of a member: register subset extraction, kit planning, cost reporting, concurrent by-election support (up to 10 simultaneous), and short-notice (<21-day) detection with an accelerated workflow under Elections Act §24.

## Contents

1. What M50 does (vs M48 recall and M51 declaration)
2. Data plane (schema)
3. Trigger types and lifecycle
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M50 does (vs M48 recall and M51 declaration)

- **M48** = recall petition and recall election mechanics; it emits `eel.governance.recall_threshold_met.v1` which M50 consumes to create a by-election for the vacated seat.
- **M51** = result declaration for any contest, including by-elections; M50 feeds the contest definition to M51.
- **M50** = all operational preparation for the by-election: voter register subset, kit plan, cost estimate, officer dispatch coordination, and legal timeline tracking. It bridges the vacancy trigger to the operational execution.

A short-notice by-election (<21 days from the triggering event to the statutory election date) is detected automatically; the system flags it as `SHORT_NOTICE = true` and routes it to the Commission Chair for expedited authorisation.

## 2. Data plane

Four aggregates (per `services/governance-svc/crates/by-election/src/models.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **ByElection** | Root entity: `by_election_id` (UUID7), `constituency_code`, `contest_type` (`MP / SENATOR / GOVERNOR / WOMEN_REP / MCA`), `trigger` (`RECALL / COURT_ORDER / DEATH / INCAPACITY`), `trigger_ref` (recall petition_id or court order ref), `trigger_date`, `statutory_election_date`, `short_notice` (boolean), `status` (`PLANNING / ACTIVE / DECLARED / CLOSED`), `declaration_id` (FK to M51). | A |
| **RegisterSubset** | Voter register extract for the constituency: `subset_id` (UUID7), `by_election_id`, `constituency_code`, `snapshot_root_hash` (M35 RegisterSnapshot reference), `registered_voters_count`, `generated_at`, `subset_hash`. | A |
| **KitPlan** | Equipment allocation plan: `plan_id` (UUID7), `by_election_id`, `bvi_kit_count`, `rts_kit_count`, `power_pack_count`, `kit_allocation[]` (`{station_code, kit_serial}`), `approved_by`, `approved_at`. | B |
| **CostReport** | Financial tracking: `report_id` (UUID7), `by_election_id`, `estimated_total_kes`, `actual_total_kes`, `line_items[]` (`{category, amount_kes}`), `submitted_to_treasury_at`, `treasury_ref`. | B |

## 3. Trigger types and lifecycle

| Trigger | Source event | Short-notice threshold |
|---|---|---|
| Recall | `eel.governance.recall_threshold_met.v1` from M48 | If recall election date < 21 days from petition threshold meeting date |
| Court order | Commission legal manually inputs court order ref | If court order specifies date < 21 days hence |
| Death | HR/payroll system (M28) or Commissioner manual input | If statutory date < 21 days from death notification |
| Incapacity | Commissioner manual input with medical certificate ref | Assessed case-by-case |

Short-notice detection:
```
short_notice = (statutory_election_date - trigger_date).days < 21
```

Short-notice by-elections skip the standard 60-day preparation window and trigger an expedited planning flow with daily Commission Chair sign-off checkpoints.

## 4. Pipeline

```
trigger event received (recall / court order / death)
      -> ByElection created (status = PLANNING)
      -> short_notice computed and flagged
      -> if SHORT_NOTICE: alert to Commission Chair + expedited workflow
      -> RegisterSubset generated:
            -> M35 RegisterSnapshot queried for constituency
            -> subset extracted (registered voters in the constituency only)
            -> subset_hash computed; stored with snapshot_root_hash reference
      -> KitPlan generated:
            -> M20 inventory queried for available kit in depot nearest to constituency
            -> kit allocated per station_code based on registered_voter_count / 500 formula
            -> plan submitted to Commission logistics for approval
            -> on approval: emit eel.inventory.kit_required.v1 -> M20 dispatches
      -> CostReport drafted:
            -> line items: officer per diem, kit transport, printing, security
            -> estimated_total_kes computed
            -> submitted to Treasury (submitted_to_treasury_at set)
      -> ByElection status -> ACTIVE
      -> election day: M10 rts-backend captures tally; M06 BVI verifies voters
      -> M51 declaration triggered; declaration_id FK set
      -> actual cost reconciled in CostReport; actual_total_kes updated
      -> status -> DECLARED -> CLOSED (on all post-election reconciliation complete)
      -> emit eel.governance.by_election_declared.v1
```

Concurrent by-elections: up to 10 simultaneous `ByElection` records in `ACTIVE` status are supported. Kit allocation is de-conflicted by M20 inventory reservation; if kit is insufficient for >10 concurrent by-elections, ops are alerted and a kit procurement emergency is escalated.

## 5. API surface

Contract: `packages/openapi-contracts/governance/governance-by-election.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/by-elections` | Commission officer (JWT) | Create ByElection manually (death / incapacity / court order) |
| `GET /v1/by-elections` | Commission admin | List by-elections; filter by status / trigger |
| `GET /v1/by-elections/{by_election_id}` | Commission admin or public | By-election detail |
| `GET /v1/by-elections/{by_election_id}/register-subset` | Commission officer | RegisterSubset detail + hash |
| `POST /v1/by-elections/{by_election_id}/kit-plan` | Logistics officer | Submit KitPlan |
| `PUT /v1/by-elections/{by_election_id}/kit-plan/approve` | Commission Director | Approve KitPlan |
| `PUT /v1/by-elections/{by_election_id}/activate` | Commissioner + HSM | Activate by-election |
| `GET /v1/by-elections/{by_election_id}/cost-report` | Treasury officer | CostReport detail |
| `PATCH /v1/by-elections/{by_election_id}/cost-report` | Finance officer | Update actual cost line items |
| `GET /v1/by-elections/public` | Public (60/hr) | Active by-elections with statutory dates |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| By-election admin console (Django) | `services/governance-svc/django/governance-admin` | Create + manage by-elections; Commissioner activates |
| Operations planning console (LiveView) | `services/governance-svc/ex/court_web` (shared LiveView) | Logistics officers plan kit + register subset |
| Public notice portal (Astro) | `services/governance-svc/astro/governance-public` | Citizens view by-election dates + constituency |

Languages: EN + SW on all surfaces. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **RegisterSubset immutability** - `RegisterSubset.subset_hash` is computed over the serialised subset and stored alongside the M35 `snapshot_root_hash`. If a post-creation register change is claimed to affect the by-election, the subset hash can be verified against the original snapshot to demonstrate whether the change was included or not.
- **Trigger provenance** - `ByElection.trigger_ref` stores the M48 `petition_id` (for recalls) or the court order reference (for court-ordered vacancies). The trigger provenance is HSM-audited.
- **Kit de-confliction** - M20 inventory reservation is backed by a PostgreSQL advisory lock per `kit_serial`. Concurrent kit plan submissions cannot double-allocate the same kit.
- **Short-notice audit** - `short_notice = true` records are flagged in the audit dashboard with the computed days-to-election. Commission Chair sign-off is required within 2 hours of flagging; the system alerts if sign-off is not received.
- **Cost report Treasury linkage** - `CostReport.treasury_ref` must be set before the by-election can be closed. A by-election without a treasury submission cannot move to `CLOSED` status.

## 8. Kenya statute alignment

- **Elections Act §24** - the Commission must conduct a by-election within 90 days of a vacancy (with exceptions for <3 months to a general election). M50 tracks the statutory_election_date and alerts if the date would exceed the 90-day limit.
- **Constitution of Kenya 2010, Art. 104(3)** - recall success triggers a by-election; M50 operationalises the handoff from M48.
- **Elections (General) Regulations 2012, Reg 4** - the Commission must gazette a by-election at least 21 days in advance; `short_notice = true` triggers compliance review.
- **Public Finance Management Act §53** - cost reports for the Commission expenditure must be submitted to the National Treasury; `CostReport.submitted_to_treasury_at` tracks compliance.

## 9. Honest caveats + external prerequisites

- Court-ordered vacancies require the court order to be received and entered by Commission legal staff. There is no eCourt API integration for by-election triggers (unlike M40 evidence bundles). Manual entry is the intake channel.
- Kit sufficiency for 10 concurrent by-elections is a planning assumption. The actual kit pool is managed by M20; if multiple by-elections are triggered simultaneously after a disputed general election result, kit availability may be constrained.
- Cost report `actual_total_kes` is updated manually by finance staff as invoices are received. The system tracks estimated vs. actual but cannot automate invoice ingestion from suppliers.
- The 90-day statutory deadline is advisory within the system. M50 alerts if the date is at risk but cannot compel Commission action. The Commission must make the operational decision.

## 10. Test coverage + operational runbooks

Tests under `services/governance-svc/crates/by-election/tests/`: short-notice detection arithmetic, concurrent by-election kit de-confliction, register subset hash computation, cost report lifecycle integration.

Runbooks under `docs/runbooks/`:

- `governance-svc.md` - by-election module deploy + rollback.
- `short-notice-by-election.md` - expedited planning procedure for <21-day by-elections.
- `concurrent-by-elections.md` - kit allocation protocol for >5 simultaneous by-elections.
- `cost-report-submission.md` - Treasury submission procedure.

Related documents: spec at `docs/plans/modules/by-election-operations.md`; M48 at `docs/features/recall.md`; M51 at `docs/features/result-declaration-certification.md`; capability catalogue at `services/governance-svc/docs/CAPABILITIES.md`.

---

*Module documentation version: 1.0. Last updated: 2026-08-22. Review before each election cycle to verify crate paths, OpenAPI contract versions, integration endpoint URLs, and statutory rule offsets remain current.*

**Performance targets**: All read endpoints respond within 200ms at p95 under 100 concurrent users. Write endpoints (including HSM-signing paths) respond within 1,000ms at p95. Bulk operations (roster upload, CSV import) are async with a job status poll endpoint.
