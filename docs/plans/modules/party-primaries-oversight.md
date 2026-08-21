# M46 party-primaries-oversight

| field | value |
|---|---|
| M-number | M46 |
| Home service | crms-svc (ADR-0020) |
| Owning work package | WP-4.11 (docs/plans/05-wave-4-operations.md) |
| Size | L |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M46-party-primaries-oversight.md |

## Purpose

Party primaries are the single highest-fraud-risk event in the Kenyan electoral cycle (Strategy §XXI.A.1). IEBC has no constitutional mandate to run primaries but has an oversight function to receive and publish results and adjudicate complaints. This module provides: a reduced-feature KIEMS-lite kit for parties that opt in; a primary-specific integrity dashboard for IEBC oversight; primary-result publication with Merkle-anchored integrity; and primary-dispute intake routed to PPDT and the Political Parties Liaison Committee. The module is a pro-bono offering distinct from the IEBC tender scope; IPR vests in IEBC per GCC 15.4, and Datacraft retains the right to productise separately.

Strategy reference: §XXI.A.1 (Party Primaries). Political Parties Act 2011 s.22; Elections Act s.33.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Party Opt-In and Result Upload Portal | Astro | services/crms-svc/astro/primaries/ |
| IEBC Oversight Dashboard | Phoenix LiveView | services/crms-svc/ex/primaries_web/ |
| Public Primary Results Portal (WCAG 2.2 AA) | Astro | services/crms-svc/astro/primaries/ |
| Candidate Dispute Portal (WCAG 2.2 AA) | Astro | services/crms-svc/astro/primaries/ |
| API | Rust (axum) | services/crms-svc/crates/crms-api/ |

Languages: EN + SW; public portal WCAG 2.2 AA.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| PrimaryOptIn | id, party_id (FK to M45 PoliticalParty), election_cycle, mou_signed_at, approved_by, approved_at | B | T1-consensus |
| PrimaryResult | id, party_id, constituency_code, position, submitted_at, submitted_by, candidate_tallies (list of {candidate_name, votes}), spoilt_ballots, total_turnout, result_hash (SHA-256 of canonical JSON), merkle_leaf_id, published_at, amended (bool), amendment_reason | A | T1-audit |
| PrimaryDispute | id, result_id, complainant_name, complainant_id, grounds, evidence_urls, status (SUBMITTED/PDM_REVIEW/PPDT_REFERRED/HEARING/DETERMINATION/WITHDRAWN), pdm_submitted_at, ppdt_referral_at, ppdt_case_number, determination_url, created_at, updated_at | B | T1-consensus |
| OversightNotice | id, party_id, constituency_code, grounds, issued_by, issued_at, published_at, merkle_leaf_id | A | T1-audit |

## APIs

OpenAPI file: `packages/openapi-contracts/crms/primaries.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/opt-in | 2 | JWT+SpiceDB (PARTY_OFFICIAL) | yes | party opts into KIEMS-lite oversight; M45 ACTIVE status gate enforced |
| POST /v1/results | 2 | JWT+SpiceDB (PARTY_RETURNING_OFFICER) | yes | submit constituency primary result; Merkle receipt in response; <= 2 s |
| GET /v1/results | 1 | public | n/a | public results listing (paginated; party + constituency + position filter) |
| GET /v1/results/{id} | 1 | public | n/a | single constituency result with Merkle path |
| PATCH /v1/results/{id} | 2 | JWT+SpiceDB (PARTY_RETURNING_OFFICER) | yes | request amendment within 48 hr; IEBC Oversight Officer approves; original retained |
| POST /v1/disputes | 2 | public | yes | submit primary dispute; PDM clock started |
| GET /v1/disputes/{id} | 2 | JWT+SpiceDB (COMPLAINANT/PPDT_CLERK) | n/a | dispute status with timeline |
| POST /v1/oversight/notices | 2 | JWT+SpiceDB (OVERSIGHT_OFFICER) | yes | IEBC issues oversight notice; published to public portal |
| GET /v1/oversight/dashboard | 2 | JWT+SpiceDB (OVERSIGHT_OFFICER) | n/a | constituency map: submission status, anomaly flags, complaint density |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.crms.primary_result_submitted.v1 | emits | eel.crms.primaries.v1 | audit | yes | at-least-once idempotent |
| eel.crms.primary_result_published.v1 | emits | eel.crms.primaries.v1 | audit | yes | at-least-once idempotent |
| eel.crms.primary_dispute_submitted.v1 | emits | eel.crms.primaries.v1 | operational | no | at-least-once idempotent |
| eel.crms.primary_dispute_ppdt_referred.v1 | emits | eel.crms.primaries.v1 | audit | yes | exactly-once |
| eel.crms.oversight_notice_issued.v1 | emits | eel.crms.primaries.v1 | audit | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M45 Party Registration | internal | integrations/party-registration/ | yes | ACTIVE status gate for opt-in; status polled on each opt-in request |
| M12 CRMS | internal | integrations/crms/ | yes | primary winners surfaced for nomination confirmation; queue on failure |
| PPDT | external | integrations/ppdt/ | yes (stub) | dispute referral package; determination ingest; fallback to structured PDF email |
| P05 Merkle Log | P05 | integrations/merkle/ | yes | result hash and oversight notice anchoring |
| M44 Turnout Forecasting | internal | integrations/turnout/ | yes | primary turnout data as historical baseline for M44 model; non-critical |
| M16 Election Command Centre | internal | integrations/ecc/ | yes | primary oversight layer during primary period; non-critical |
| M20 Inventory Management | internal | integrations/inventory/ | yes | KIEMS-lite kit tracking; kit serial binding per constituency |
| M43 Observer Accreditation | internal | integrations/observer-accreditation/ | yes | accredited observers may attend primaries; no data consumed; reference only |

## ML and AI

- Turnout anomaly detection: constituency-level primary turnout compared against party's historical primary turnout (where available); deviation > 2 SD flagged on the Oversight Dashboard. Rule-based, not a separate ML model; M28 not involved.
- Complaint density heat map: choropleth derived from dispute count per constituency; no ML.

## Dependencies

- Modules: M12, M16, M20, M43, M44, M45
- Platform: P05
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs

## Feature checklist

1. FR-M46-001: party opt-in; registered party (ACTIVE in M45) applies; IEBC Oversight Officer approves; party signs MOU committing to result publication and IEBC oversight
2. FR-M46-002: KIEMS-lite kit configuration: voter-roll import (party-supplied CSV), biometric or PIN authentication (party choice), ballot casting, result tallying; no ABIS integration; offline-capable; no national network connectivity required
3. FR-M46-003: voter roll supplied by party per constituency (party members eligible to vote); format validated but not CVR-cross-referenced (party-member data outside IEBC mandate)
4. FR-M46-004: kit serial bound to constituency code and primary election date; cannot be used at another constituency
5. FR-M46-005: result submission via signed JSON upload; party-provisioned certificate verified; Merkle receipt returned within 2 minutes
6. FR-M46-006: results published to public portal within 4 hours of submission; per-constituency breakdown; CSV/JSON download
7. FR-M46-007: each result hash-anchored in P05 Merkle chain; public Merkle path verifiable
8. FR-M46-008: result amendment within 48 hours with supporting documentation; IEBC Oversight Officer approves; original and amended versions both retained
9. FR-M46-009: Oversight Officer dashboard: constituency map with submission status (SUBMITTED/PENDING/LATE/DISPUTED); turnout anomaly flags (> 2 SD from historical primary turnout)
10. FR-M46-010: complaint density heat map: disputes per constituency as choropleth; hot-spots highlighted
11. FR-M46-011: IEBC oversight notice: Oversight Officer issues formal notice to party for specific constituency irregularities; notice published on public portal alongside those constituency results
12. FR-M46-012: candidate dispute submission within 48 hours of result publication; structured form (constituency, grounds, evidence)
13. FR-M46-013: PDM first-step attestation required; if PDM unresolved within 7 days, dispute auto-escalates to PPDT; 7-day deadline enforced by system; non-overridable
14. FR-M46-014: PPDT referral package (result PDF, dispute form, PDM response) generated and dispatched within 24 hours of PDM escalation; PPDT case number returned
15. FR-M46-015: dispute status tracker: SUBMITTED -> PDM_REVIEW -> PPDT_REFERRED -> HEARING -> DETERMINATION/WITHDRAWN; candidates and parties track lifecycle
16. FR-M46-016: determination-driven result correction: PPDT determination mandates correction; Oversight Officer updates published result with determination reference; original archived; Merkle proof of both versions retained
17. KIEMS-lite kit: fully offline; result upload batch on connectivity restoration; kit tracked in M20
18. Party-member voter rolls not retained beyond tallying session; only aggregate results retained by IEBC; DPA 2019 DPIA registered

## Acceptance criteria

1. AC-M46-01: party opts in and signs MOU; KIEMS-lite kit provisioned; result submitted offline and batch-uploaded on connectivity restore; Merkle receipt verified; UAT
2. AC-M46-02: result published to public portal within 4 hours of submission; Merkle hash on portal matches submitted hash; integration test
3. AC-M46-03: dispute submitted; PDM clock runs 7 days; auto-escalation to PPDT; structured referral package dispatched; PPDT stub test
4. AC-M46-04: oversight notice issued by IEBC Oversight Officer; appears on public portal alongside constituency results within 1 hour; UAT
5. AC-M46-05: PPDT determination received; published result updated with determination reference; original archived; both Merkle proofs verifiable; integration test
6. AC-M46-06: M45 ACTIVE status gate prevents non-ACTIVE party from opting in; unit test

## Applicable edge cases

**connectivity**
- EC-CON-01: KIEMS-lite kit offline during primary; local result stored; batch-uploaded on reconnect; Merkle receipt issued on arrival; no duplicate submission

**security**
- EC-SEC-08: party submits fraudulent result; result hash and Merkle anchoring make post-hoc tampering detectable; observer presence encouraged via M43; Oversight Officer can issue oversight notice and request amendment
- EC-SEC-01: result submission with invalid party-provisioned certificate; rejected with 401; logged

**operations**
- EC-OPS-05: PPDT overwhelmed by volume of primary disputes; structured intake reduces manual work; PPDT capacity planning based on opt-in party count and primary constituency count
- EC-OPS-06: PDM stage used to delay legitimate disputes; 7-day hard PDM deadline enforced by system; auto-escalation non-overridable

**legal**
- EC-LEG-02: party-member voter rolls not retained post-tally; IEBC retains only aggregate results; DPA 2019 DPIA filed; data minimisation enforced at tallying session end
- EC-LEG-04: low opt-in rate means many party primaries occur without KIEMS-lite; result publication obligation applies to all parties per PPA 2011 s.22 regardless of opt-in; IEBC Oversight Officer can request result submission from any party

**scale**
- EC-SCL-01: 200 result uploads in 2-hour window at primary close; crms-svc horizontally scaled; Merkle anchor pipeline handles burst

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Merkle receipt generation, result hash, PDM 7-day timer, ACTIVE status gate, amendment version control | nextest | M46-UNIT- |
| contract | M45 status gate, P05 hash anchor, PPDT referral API, M12 primary winner ingestion, M20 kit tracking | Pact | M46-PACT- |
| integration | opt-in, result submission, publication, dispute, PDM escalation, PPDT referral, determination correction | integration_test | M46-INT- |
| e2e | party opts in, kit provisioned, result uploaded offline, published, disputed, PPDT referred, determination corrects result | nextest e2e | M46-E2E- |
| security | invalid party certificate (reject); result hash tamper detection; non-ACTIVE opt-in (deny) | ZAP + red team | M46-SEC- |
| a11y | WCAG 2.2 AA on public results portal and candidate dispute portal | axe-core + manual | M46-A11Y- |
| offline | KIEMS-lite kit network-loss simulation; batch upload on reconnect; no duplicate | netem | M46-OFFLN- |
| load | 200 concurrent result uploads in 2-hour window | k6 | M46-LOAD- |

Tags: `#[trace("FR-M46-013","AC-M46-03","EC-OPS-06")]`

## Runbooks and dashboards

- `docs/runbooks/crms-svc-degradation.md` - extend with primary result submission failures and Merkle anchor queue drain
- `docs/runbooks/primary-dispute-escalation.md` - new runbook; PDM escalation SLA; PPDT referral dispatch; oversight notice procedure
- `docs/runbooks/kiems-lite-kit-recovery.md` - new runbook; kit serial conflict resolution; offline result batch upload; M20 kit tracking update
- Grafana dashboard: `eel-iems-46` (alert ruleset: `iems-46-alerts`)
- SLO alerts: result published within 4 hours of submission; PPDT referral dispatch within 24 hours of PDM escalation; public portal p99 <= 2 s

## Open questions

- PPDT API MoU: same dependency as M45; if not signed by T-300, email-PDF fallback is the production path through the primary window; PPDT staff must be pre-trained on the structured referral format.
- KIEMS-lite kit firmware: a subset of M05 BVI functionality must be extracted into a standalone firmware build; M05 team must confirm build timeline (target T-210); this is a hard dependency.
- Pro-bono scope boundary: M46 is explicitly outside the IEBC tender scope; any IEBC request to extend KIEMS-lite functionality (e.g. ABIS integration) requires a separate change-control event and Datacraft sign-off.
- Party opt-in outreach: expected opt-in rate is uncertain; the public-portal result-publication path (PPA 2011 s.22) applies to all parties regardless of opt-in; IEBC Oversight Officer must have a manual result-upload channel for non-opted-in parties to submit results in the standard format.
