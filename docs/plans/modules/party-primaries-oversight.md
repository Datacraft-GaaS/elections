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

KIEMS-lite opt-in and provisioning:
1. FR-M46-001: party opt-in: registered ACTIVE party (M45 status gate checked on each request) applies; IEBC Oversight Officer approves; party signs MOU committing to result publication and IEBC oversight
2. FR-M46-002: MOU signing via authenticated portal; MOU document stored; mou_signed_at timestamp recorded; party cannot submit results without approved opt-in
3. FR-M46-003: KIEMS-lite kit configuration: voter-roll import (party-supplied CSV), biometric or PIN voter authentication (party choice), ballot casting, result tallying; no ABIS integration; offline-capable; no national network connectivity required
4. FR-M46-004: voter roll supplied by party per constituency (party members eligible to vote); CSV validated (column schema, UTF-8 encoding, max 50,000 rows) but not CVR-cross-referenced
5. FR-M46-005: voter roll upload error report: per-row format errors listed; valid rows accepted; party may re-upload corrected rows before primary date
6. FR-M46-006: kit serial bound to constituency code and primary election date; cannot be used at another constituency; binding signed and recorded in M20 inventory
7. FR-M46-007: kit provisioning tracked in M20 inventory management; kit serial, constituency code, handover timestamp, and custodian recorded
8. FR-M46-008: party Returning Officer credentials provisioned per kit; credentials scoped to constituency and primary date; cannot be reused across events

Result submission:
9. FR-M46-009: result submission via signed JSON upload; party-provisioned certificate verified against stored public key; rejected with 401 on certificate mismatch; event logged; Oversight Officer alerted
10. FR-M46-010: result payload must include candidate tallies (list of {candidate_name, votes}), spoilt_ballots, total_turnout, constituency_code, position, submitted_at
11. FR-M46-011: Merkle receipt returned within 2 minutes of submission; receipt includes result_hash (SHA-256 of canonical JSON) and merkle_leaf_id
12. FR-M46-012: idempotent result submission: duplicate submission with same constituency_code, position, party_id, and primary date returns cached Merkle receipt; no second DB write
13. FR-M46-013: result amendment within 48 hours of publication with supporting documentation; IEBC Oversight Officer approves amendment; original version retained; amendment_reason recorded
14. FR-M46-014: both original and amended result versions retained; amendment chain visible on Oversight Officer dashboard and public portal

Publication:
15. FR-M46-015: results published to public portal within 4 hours of submission; per-constituency breakdown; CSV and JSON download available
16. FR-M46-016: result publication event emitted (eel.crms.primary_result_published.v1); M12 CRMS receives primary winners for nomination confirmation
17. FR-M46-017: each result hash-anchored in P05 Merkle chain; public Merkle path verifiable via portal verifier endpoint
18. FR-M46-018: public portal shows per-constituency results with party, position, candidate tallies, spoilt ballots, total turnout; oversight notice banner shown where issued
19. FR-M46-019: results downloadable as CSV and JSON; download includes result_hash for independent verification

Oversight dashboard:
20. FR-M46-020: Oversight Officer dashboard: constituency map with submission status per constituency (SUBMITTED / PENDING / LATE / DISPUTED)
21. FR-M46-021: turnout anomaly flags: > 2 SD deviation from party historical primary turnout flagged; historical baseline sourced from M44 Turnout Forecasting
22. FR-M46-022: complaint density heat map: disputes per constituency displayed as choropleth; hot-spots highlighted for Oversight Officer attention
23. FR-M46-023: Oversight Officer can filter dashboard by party, county, position, and anomaly flag
24. FR-M46-024: Oversight Officer receives alert when constituency result is LATE (> 6 hours post-primary-close without submission)
25. FR-M46-025: IEBC oversight notice: Oversight Officer issues formal notice to party for specific constituency irregularities; mandatory structured reason field
26. FR-M46-026: oversight notice Merkle-anchored in P05; published to public portal alongside constituency results within 1 hour of issuance; eel.crms.oversight_notice_issued.v1 emitted
27. FR-M46-027: Oversight Officer can retract oversight notice with reason; original notice retained; retraction recorded; retraction published on public portal

Dispute intake:
28. FR-M46-028: candidate dispute submission within 48 hours of result publication; structured form: constituency, position, grounds, supporting evidence URLs
29. FR-M46-029: dispute form requires attestation that candidate first referred dispute to party internal dispute mechanism (PDM); PDM attestation timestamp recorded
30. FR-M46-030: PDM 7-day hard deadline enforced by system scheduler; if PDM unresolved at day 7, auto-escalation to PPDT triggered; non-overridable by any user role
31. FR-M46-031: PPDT referral package generated automatically on PDM escalation: result PDF, dispute form, PDM chronology; package stored against dispute record
32. FR-M46-032: PPDT referral package dispatched within 24 hours of PDM escalation; eel.crms.primary_dispute_ppdt_referred.v1 emitted; dispatch timestamp logged
33. FR-M46-033: PPDT case number stored on dispute record within 24 hours of referral; eel_ppdt_case_number field set
34. FR-M46-034: dispute status tracker: SUBMITTED -> PDM_REVIEW -> PPDT_REFERRED -> HEARING -> DETERMINATION / WITHDRAWN; candidates and parties track via portal
35. FR-M46-035: determination-driven result correction: PPDT determination mandates correction; Oversight Officer updates published result with determination reference; original archived; Merkle proof of both versions retained
36. FR-M46-036: PPDT determination ingest: signed PDF uploaded by PPDT Clerk; stored against dispute record; dispute status updated to DETERMINATION

Offline and connectivity:
37. FR-M46-037: KIEMS-lite kit fully offline during primary; local result stored with digital signature; batch-uploaded on connectivity restoration; no duplicate submission on reconnect
38. FR-M46-038: kit offline duration tracked; if offline > 48 hours post-primary-close, Oversight Officer alerted and constituency marked LATE on dashboard
39. FR-M46-039: offline result stored on kit with signing key; transmission includes original signed blob; Merkle receipt issued on backend arrival

Access control and security:
40. FR-M46-040: result submission endpoint authenticated with party-provisioned certificate; certificate tied to party_id, constituency_code, and primary date; not JWT
41. FR-M46-041: public results portal: no authentication required; rate-limited by IP/64; WCAG 2.2 AA; EN + SW
42. FR-M46-042: candidate dispute portal: public submission; rate-limited; WCAG 2.2 AA; EN + SW
43. FR-M46-043: all dispute evidence uploads virus-scanned and format-validated before acceptance; malformed or infected files rejected with structured error

Data protection and retention:
44. FR-M46-044: party-member voter rolls not retained beyond tallying session; only aggregate results retained by IEBC; deletion confirmed within 1 hour of session end; DPIA registered
45. FR-M46-045: public results portal contains no party-member PII; only candidate names, vote tallies, and aggregate turnout published
46. FR-M46-046: all audit events appended to P05 Merkle chain: result submission, publication, amendment, dispute submission, PPDT referral, determination, oversight notice

Availability and SLAs:
47. FR-M46-047: system availability >= 99.5% during primary result submission windows (T-90 to T-75)
48. FR-M46-048: result submission p99 upload response <= 2 s; Merkle receipt returned within 2 minutes of submission
49. FR-M46-049: public portal p99 <= 2 s under 20,000 concurrent users
50. FR-M46-050: PPDT referral dispatch within 24 hours of PDM escalation trigger

Observability:
51. FR-M46-051: Grafana dashboard `eel-iems-46` exists; SLO alerts for result publication <= 4 hours, PPDT referral <= 24 hours, portal p99 <= 2 s; alert ruleset `iems-46-alerts` in Alertmanager
52. FR-M46-052: OpenTelemetry traces exported to central collector; trace IDs included in Merkle receipt responses for correlation

## Acceptance criteria

1. AC-M46-01: M45 ACTIVE status gate prevents non-ACTIVE (SUSPENDED, DEREGISTERED) party from opting in; 403 returned; unit test with each non-ACTIVE status
2. AC-M46-02: result submitted offline by KIEMS-lite kit; batch-uploaded on connectivity restore; Merkle receipt issued on arrival; no duplicate submission; offline netem integration test passes
3. AC-M46-03: result published to public portal within 4 hours of submission; end-to-end integration test measures publication latency
4. AC-M46-04: result_hash on public portal matches SHA-256 of canonical submitted JSON; deterministic hash property test with 50 synthetic results
5. AC-M46-05: dispute submitted; PDM clock runs exactly 7 days; auto-escalation fires at day 7; PPDT referral package dispatched within 24 hours; PPDT stub integration test
6. AC-M46-06: oversight notice issued by Oversight Officer; appears on public portal within 1 hour; Merkle-anchored; integration test
7. AC-M46-07: PPDT determination received; published result updated with determination reference; original archived; both Merkle proofs independently verifiable; integration test
8. AC-M46-08: result submission p99 <= 2 s under 200 concurrent uploads in 2-hour window; k6 load test passes
9. AC-M46-09: public portal p99 <= 2 s under 20,000 concurrent users; k6 load test passes
10. AC-M46-10: system availability >= 99.5% during primary result submission window (T-90 to T-75); SLO burn-rate alert fires at > 0.1% error budget per hour
11. AC-M46-11: WCAG 2.2 AA axe-core audit passes on public results portal and candidate dispute portal; EN + SW strings verified; manual review sign-off
12. AC-M46-12: party-member voter rolls deleted within 1 hour of tallying session end; integration test measures deletion timestamp against session_end_at
13. AC-M46-13: duplicate result submission returns cached Merkle receipt; no second DB write; idempotency integration test with same constituency + party + position + date
14. AC-M46-14: PPDT referral dispatch <= 24 hours of PDM escalation trigger; timer integration test with mocked clock

## Applicable edge cases

connectivity:
- EC-CON-01: KIEMS-lite kit offline during primary; local result stored with digital signature; batch-uploaded on reconnect; Merkle receipt issued on arrival; no duplicate submission on reconnect
- EC-CON-05: party result upload link flapping; exponential backoff with retry budget; no duplicate result; server dedupe count 0

data:
- EC-DAT-01: duplicate result submission with same constituency, party, position, and date; cached Merkle receipt returned; no second DB write; no second Merkle leaf created
- EC-DAT-02: same submission idempotency key with different candidate tally body; 422 idempotency-key-reuse; original result unchanged
- EC-DAT-07: partial batch in voter-roll CSV upload; per-row error report returned; valid rows accepted; invalid rows listed; party re-uploads corrections before primary date
- EC-DAT-09: Unicode candidate names in result JSON; NFC normalisation applied; correct display on public portal in EN and SW
- EC-DAT-10: zero candidate tallies (all spoilt ballots constituency); no divide-by-zero in turnout percentage; valid result stored and published

security:
- EC-SEC-01: result submission with invalid party-provisioned certificate; rejected with 401; event logged; Oversight Officer alerted
- EC-SEC-05: injection on grounds, candidate_name, or constituency_code fields in dispute or result submissions; parameterised queries only; Semgrep and ZAP pass
- EC-SEC-08: party submits fraudulent result; result_hash and Merkle anchoring make post-hoc tampering detectable; observer presence via M43; Oversight Officer issues oversight notice and requests amendment
- EC-SEC-16: candidate NIN and party-member voter data must not appear in logs, traces, or spans; log-scanner CI fixture passes on every build

identity:
- EC-IDN-02: party Returning Officer JWT expires mid-upload; in-flight upload completes; next request returns 401 and silent refresh; no result data lost
- EC-IDN-09: dual-control required for result amendment; approver revoked mid-flow; re-validation at commit; amendment blocked if approver invalid; compensating event logged
- EC-IDN-10: SpiceDB unavailable during Oversight Officer action; fail closed; 503 returned; no oversight notice persisted in partial state

scale:
- EC-SCL-01: 200 result uploads in 2-hour window at primary close; crms-svc horizontally scales; p99 <= 2 s maintained; Merkle anchor pipeline handles burst
- EC-SCL-04: Kafka consumer lag > 60 s during result burst; backpressure applied; Merkle anchor pipeline drains without OOM; no result lost

legal:
- EC-LEG-02: court-ordered injunction on a specific primary result; Oversight Officer places result in LEGAL_HOLD status; public portal displays pending-legal-challenge notice; result not amended without court resolution
- EC-LEG-04: low opt-in rate; parties not using KIEMS-lite still obligated under PPA 2011 s.22; Oversight Officer uses manual result upload channel; results ingested via same API with manual flag; audit-logged

operations:
- EC-OPS-05: PPDT API unavailable; fallback to structured PDF email; dispute record marked PPDT_FALLBACK; case number manually entered by IEBC staff after PPDT responds
- EC-OPS-06: PDM 7-day deadline used by parties to delay legitimate disputes; hard deadline enforced by system scheduler; auto-escalation non-overridable; PPDT alerted

human:
- EC-HUM-08: EN and SW UI strings on public results portal; Swahili candidate names with long strings; no truncation; 150% pseudo-locale test passes
- EC-HUM-06: screen reader support on candidate dispute portal; full task completion via keyboard and ARIA; axe-core audit passes

time:
- EC-TIM-04: primary date stored in UTC; display converted to EAT on portal; DST handling correct; timezone snapshot test passes
- EC-TIM-08: PDM 7-day escalation deadline computed in EAT; boundary test +/- 1 second; escalation fires at correct EAT midnight timestamp

ML:
- EC-ML-03: KIEMS-lite biometric capture on degraded sensor (dust, humidity); quality gate routes to PIN fallback; no denial of vote; UAT scenario covered

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
