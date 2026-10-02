# M50 by-election-operations

| field | value |
|---|---|
| M-number | M50 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.2 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M50-by-election-operations.md |

## Purpose

Manages the full lifecycle of by-elections triggered by recall (M48), vacancy, or court order, supporting up to 10 concurrent by-elections with register subsets, kit plan, cost reporting, and the short-notice (<21-day) notice flow under Elections Act s.24.

## Surfaces and languages

| surface | technology | path under services/governance-svc/ |
|---|---|---|
| Admin by-election configuration | Django | django/governance-admin/ |
| Public by-election notice portal | Astro | astro/governance-public/ |
| Internal API | Rust (axum) | crates/by-election/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| ByElection | id, constituency_id, trigger (recall/vacancy/court), status (notice/planning/active/closed/declared), notice_date, election_date, cost_ceiling_kes | C | T1-consensus |
| RegisterSubset | id, by_election_id, snapshot_id (FK to M02), station_count, voter_count, generated_at | C | T1-consensus |
| KitPlan | id, by_election_id, station_assignments, kit_ids, signed_at | C | T1-consensus |
| CostReport | id, by_election_id, budget_lines, actual_expenditure, variance, submitted_at | C | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/by-elections | 1 signed-static | public | n/a | list by-elections with status |
| GET /v1/by-elections/{id} | 2 live-per-user | public | n/a | by-election detail |
| POST /v1/by-elections | 3 accredited | JWT+SpiceDB | yes | create by-election (Commissioner dual-sign) |
| POST /v1/by-elections/{id}/register-subset | 3 accredited | JWT+SpiceDB | yes | generate register subset from M02 |
| POST /v1/by-elections/{id}/kit-plan | 3 accredited | JWT+SpiceDB | yes | create and sign kit plan |
| PUT /v1/by-elections/{id}/activate | 3 accredited | JWT+SpiceDB | yes | activate; triggers M04 register subset notice |
| POST /v1/by-elections/{id}/cost-report | 3 accredited | JWT+SpiceDB | yes | submit cost report |
| GET /v1/by-elections/{id}/cost-report | 3 accredited | JWT+SpiceDB | n/a | cost report |

OpenAPI: packages/openapi-contracts/governance/by-election-operations.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.by_election_created.v1 | emits | eel.governance.by_election.v1 | operational | no | at-least-once idempotent |
| eel.governance.by_election_activated.v1 | emits | eel.governance.by_election.v1 | operational | no | at-least-once idempotent |
| eel.governance.by_election_declared.v1 | emits | eel.governance.by_election.v1 | operational | no | at-least-once idempotent |
| eel.governance.recall_threshold_met.v1 | consumes | eel.governance.recall.v1 | operational | no | at-least-once idempotent |
| eel.declaration.result_declared.v1 | consumes | eel.declaration.result.v1 | event-source | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M04 VPRS (register notice) | internal | crates/by-election/ | yes | advisory; notice continues |
| M10 rts-backend (tally) | internal | crates/by-election/ | yes | circuit breaker; provisional from snapshot |
| M12 CRMS (candidate nomination) | internal | crates/by-election/ | yes | advisory; nomination window opens |
| M20 inventory-management | internal | crates/by-election/ | yes | advisory; kit allocation |
| M22 campaign-finance | internal | crates/by-election/ | yes | advisory; disclosure window |
| M51 result-declaration | internal | crates/by-election/ | yes | hard dependency for declaration |
| M52 election-calendar-gazette | internal | crates/by-election/ | yes | advisory; gazette notice |
| M55 hr-payroll (cost report link) | internal | crates/by-election/ | yes | advisory |

## ML and AI

none

## Dependencies

- Modules: M04, M10, M12, M20, M22, M51, M52, M55
- Platform: P01 (SpiceDB), P04 (Kafka outbox), P06 (HSM for kit plan signing)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs, eel-web-py

## Feature checklist

By-election creation and lifecycle:
1. FR-M50-001: Create by-election with seat type (MP/SENATOR/GOVERNOR/CWR/MCA/PRESIDENTIAL), constituency/ward code, trigger reason (DEATH/RECALL/NULLIFICATION/VACANCY), vacancy date, and polling date; Commissioner dual-sign MFA required before record is written.
2. FR-M50-002: Enforce Commissioner dual-sign approval; both signatures verified in SpiceDB before ByElection row commits; partial-sign state persists for up to 24 hours then expires.
3. FR-M50-003: By-election creation from M48 recall event is idempotent; duplicate recall events from the same recall petition produce exactly one ByElection record and one created event.
4. FR-M50-004: Status FSM enforces valid transitions DRAFT, GAZETTED, NOMINATIONS_OPEN, CAMPAIGN, POLLS_OPEN, TALLYING, DECLARED, CERTIFIED, PETITION_WINDOW, CLOSED; any invalid transition returns 422 with transition-error problem+json.
5. FR-M50-005: Rapid-activation path: create-to-GAZETTED system processing completes within 2 hours for court-nullification trigger; system clock tracks elapsed time and alerts if approaching 2-hour wall.
6. FR-M50-006: System detects when polling_date minus notice_date is less than 21 days and automatically routes the by-election through the accelerated statutory checklist; legal officer notified with list of outstanding steps.
7. FR-M50-007: Statutory timeline checklist auto-populates at creation from seat type and trigger reason; deadlines drawn from Elections Act s.11, s.22-24, s.25 and Constitution Art 101(4).
8. FR-M50-008: Support up to 10 concurrent by-elections in distinct constituencies without shared-resource deadlock; concurrency guard rejects creation of an 11th active by-election with 409.
9. FR-M50-009: Per-by-election SLA configuration: sla_rto_minutes and sla_rpo_minutes settable by Commission at creation; defaults are 30 and 5; values stored on ByElection and drive per-event SLO burn-rate alerts.
10. FR-M50-010: Audit trail records every state transition with actor identity, timestamp, dual-sign evidence hash, and IP; append-only; no update or delete path.

Statutory compliance and gazette:
11. FR-M50-011: Emit gazette notice trigger to M52 at GAZETTED transition; gazette_notice_id returned and stored on ByElection; GAZETTED transition blocked until M52 confirms notice scheduled.
12. FR-M50-012: Petition window tracked as 28-day countdown from DECLARED; deadline visible on admin and public surfaces; M52 tracks window close and fires eel.governance.by_election_petition_closed.v1 on expiry.
13. FR-M50-013: POLLS_OPEN transition blocked unless gazette_notice_id is confirmed in M52; attempt without confirmation returns 412 Precondition Failed.
14. FR-M50-014: Short-notice alert: if polling_date is fewer than 21 days from creation timestamp, legal officer receives push notification listing all outstanding statutory steps with their statutory deadlines.
15. FR-M50-015: cost_ceiling_kes stored on ByElection; alert emitted to Commission when cost report total_kes exceeds ceiling; overrun percentage included in alert payload.

Voter register subset:
16. FR-M50-016: Extract voter register subset from M04 VPRS snapshot scoped precisely to the affected constituency or ward code; extraction query uses constituency boundary polygon from M18 to exclude any out-of-scope voters.
17. FR-M50-017: Validate constituency/ward boundary against M18 boundary data before subset freeze; reject with 422 if boundary mismatch or polygon not found.
18. FR-M50-018: Freeze register subset at nomination close; write path closed via OPA policy from that timestamp; any attempted late modification returns 403 with freeze-violation reason.
19. FR-M50-019: Sign frozen register subset with HSM (P06); signed blob distributed to polling stations for the constituency; distribution confirmed per station.
20. FR-M50-020: Register subset extraction completes within 4 hours of by-election activation; timing tracked in telemetry; alert fires if 3-hour mark passes with extraction incomplete.
21. FR-M50-021: voter_count and station_count stored on RegisterSubset; count reconciled against M04 snapshot record; discrepancy > 0 raises anomaly case.
22. FR-M50-022: Register subset readable only by officers whose SpiceDB role is scoped to the affected constituency; cross-constituency read attempt returns 403.

Kit plan and deployment:
23. FR-M50-023: Kit deployment plan auto-generated from M20 inventory within 30 minutes of by-election creation; minimum kit count computed from polling station count in affected constituency.
24. FR-M50-024: Kit plan reserves kits in M20 ReservePool per active by-election; reservation blocks those kits from other by-elections until plan is cancelled or closed.
25. FR-M50-025: Sign kit plan with HSM (P06); per-station acknowledgement tracked; unacknowledged assignments listed in ECC outstanding-actions.
26. FR-M50-026: Courier manifest exportable as PDF from kit plan view; manifest includes per-kit assignment, station GPS coordinates, and expected delivery date.
27. FR-M50-027: POLLS_OPEN transition advisory check warns if any station kit assignment is unacknowledged; block is configurable; default is advisory-only with Commission override logged.
28. FR-M50-028: Alert field logistics officer and ECC when M20 reserve pool has fewer available kits than required for the by-election; shortage count included in alert.

Candidate nomination and campaign finance:
29. FR-M50-029: Open nomination window in M12 CRMS at NOMINATIONS_OPEN transition; by-election flag and constituency code set on all nominations; general-election CRMS portal reused with no separate training path.
30. FR-M50-030: Enforce same statutory nomination requirements as general election (deposit, signature count, clearance) via M12; no by-election exemptions.
31. FR-M50-031: Trigger ECFA campaign finance window in M22 at NOMINATIONS_OPEN with weekly disclosure schedule from nomination close per ECFA 2013 s.22; non-compliant candidates flagged to the Commission officer at T+3.
32. FR-M50-032: Apply per-seat ECFA ceiling tier to by-election candidates; same ceiling as general election for the same seat category; ceiling violation raises DisputeCase in M22.

Tally, declaration, and certification:
33. FR-M50-033: Route M10 RTS tally results tagged with by_election_id to constituency-scoped partition; no cross-constituency result mixing.
34. FR-M50-034: Publish results to M11 public portal on a by-election-specific page including seat type, constituency name, and vote totals; page live at TALLYING transition.
35. FR-M50-035: Trigger M51 declaration workflow at TALLYING to DECLARED transition; declaration blocked if M51 circuit breaker open; provisional status annotated.
36. FR-M50-036: Cost report submission advisory: Commission notified if cost report not submitted within T+10 days of declaration; DECLARED to CERTIFIED transition proceeds independently.
37. FR-M50-037: Tally pipeline for by-election partition meets M10 SLA: p99 ingest-to-confirmation <= 30 seconds; monitored per by-election ID.

Cost reporting:
38. FR-M50-038: Cost report stores line items: kit_days, staff_days, transport_kes, printing_kes, venue_kes, other_kes, total_kes; computed total must equal sum of components; mismatch returns 422.
39. FR-M50-039: Cost report generated and available to Finance Officer within T+14 days of declaration; automated reminder to Finance Officer at T+10 if not submitted.
40. FR-M50-040: Export cost report as OAG-ready PDF and CSV; CoB submission button triggers M54 budget module integration and sets submitted_to_cob flag with timestamp.
41. FR-M50-041: Cost report links to M55 by-election staff headcount and payroll data; staff_days derived from M55 payroll run records for the by-election period.
42. FR-M50-042: Alert Commission when actual expenditure exceeds cost_ceiling_kes; alert includes variance, percentage overrun, and link to cost report.

Observability, SLA, and reliability:
43. FR-M50-043: Per-by-election SLA dashboard displays: active by-elections list, per-event status badge, days-to-polling countdown, outstanding-actions list, and SLO burn rate.
44. FR-M50-044: OpenTelemetry traces exported to central collector; Grafana dashboard named eel-governance-50; Alertmanager ruleset eel-governance-50-alerts; both ship at OAT gate per FAT-07.
45. FR-M50-045: Circuit breakers implemented on all integration calls (M20, M51, M52, M55); each has a defined failure behaviour: advisory, queued, or blocked as specified in the integrations table.
46. FR-M50-046: Election-critical 99.99% availability maintained during active by-election period (nomination close to T+30); SLO burn-rate alert fires within 5 minutes of budget breach.
47. FR-M50-047: RTO <= 30 minutes and RPO <= 5 minutes; DR drill executed and result logged before each general-election readiness gate; Patroni fencing confirmed with no split-brain.
48. FR-M50-048: Degradation mode flag-switchable and documented in runbook; degraded state logs a DegradationEvent; reverse to normal requires dual sign-off.

Security and audit:
49. FR-M50-049: All write endpoints require JWT plus SpiceDB authorisation; SpiceDB unavailable causes fail-closed on writes and 503 problem+json; class-C reads continue.
50. FR-M50-050: Kit plan and register subset HSM signatures verified on every read; tamper event emitted to tamper topic within 30 seconds of verification failure; on-call paged.
51. FR-M50-051: Secrets and PII (voter IDs, biometric references) excluded from logs, traces, and error messages; log-scanner CI fixture passes with zero Tier A/B hits.
52. FR-M50-052: All user-supplied inputs parameterised; no dynamic SQL or shell interpolation; Semgrep compile-time check and ZAP DAST scan both clean.
53. FR-M50-053: HSM unavailable: kit plan signing queued in degraded mode; ingest continues; no data loss; queue drains when HSM recovers.
54. FR-M50-054: Idempotency key enforced on every write endpoint; same-key-same-body returns cached 2xx; same-key-different-body returns 422 idempotency-key-reuse; auto-generated tests per EC-DAT-01/02.

Events and integrations:
55. FR-M50-055: Emit eel.governance.by_election_created.v1 on creation, eel.governance.by_election_activated.v1 on POLLS_OPEN, and eel.governance.by_election_declared.v1 on declaration; all via eel-outbox-rs at-least-once idempotent delivery.
56. FR-M50-056: Consume eel.governance.recall_threshold_met.v1 from M48 topic; create by-election idempotently on receipt; counter-event fired if already exists.
57. FR-M50-057: Consume eel.declaration.result_declared.v1 from M51 exactly-once; advance status to DECLARED; trigger gazette notice via M52.
58. FR-M50-058: Events registered in Karapace; schema compatibility check passes; Pact contracts green with M48 (recall provider), M51 (declaration provider), M52 (gazette consumer).

Public portal and UI:
59. FR-M50-059: Public notice portal lists all by-elections with status, polling date, constituency name, seat type, and days-to-polling countdown; served at signed-static tier; updated within 60 seconds of status change.
60. FR-M50-060: By-election creation wizard guides user through seat type, trigger reason, dates, and statutory timeline; gazette deadlines auto-populate on seat-type selection; missing required dates shown inline.
61. FR-M50-061: By-election dashboard shows outstanding-actions list per active by-election; items past their statutory deadline highlighted with overdue badge and action owner named.

Accessibility and internationalisation:
62. FR-M50-062: Public notice portal internationalised in 11 locales; no hardcoded strings; ICU plurals; font fallback stacks for extended scripts; pseudo-locale 150% expansion test passes without truncation.
63. FR-M50-063: All voter-facing and public screens meet WCAG 2.2 AA; axe zero violations; manual screen-reader test script signed off by QA lead.

## Acceptance criteria

1. AC-M50-01: Up to 10 concurrent by-elections activate simultaneously without shared-resource deadlock; k6 load test at election-critical load confirms zero deadlocks and all read-endpoint p99 <= 500 ms throughout.
2. AC-M50-02: By-election with polling_date minus notice_date < 21 days routes automatically to accelerated statutory checklist within 60 seconds of creation; end-to-end test uses mock court-nullification trigger and asserts checklist populated with correct deadlines.
3. AC-M50-03: M48 recall event double-fired produces exactly one ByElection record and one eel.governance.by_election_created.v1 event; deduplication verified by consumer count assertion.
4. AC-M50-04: Register subset contains zero out-of-scope voter IDs; property test across 1000 randomly generated constituency polygons asserts subset voter_count equals M04 snapshot count for that polygon.
5. AC-M50-05: Register subset extraction completes within 4 hours of activation under maximum constituency load (largest constituency in M18 boundary data); timing asserted in integration test.
6. AC-M50-06: Kit deployment plan generated within 30 minutes of by-election creation; M20 reservation blocks kits from concurrent by-elections; integration test confirms zero double-allocation across 10 simultaneous by-elections.
7. AC-M50-07: M51 declaration triggered at TALLYING to DECLARED transition; end-to-end test covers full lifecycle: creation, gazette, nominations, POLLS_OPEN, TALLYING, DECLARED, gazette petition window.
8. AC-M50-08: Cost report export contains all seven line items (kit_days, staff_days, transport_kes, printing_kes, venue_kes, other_kes, total_kes) with total_kes equalling component sum; OAG-ready CSV and PDF generated without error; integration test asserts field presence and arithmetic.
9. AC-M50-09: API p99 latency <= 500 ms and p95 <= 200 ms at election-critical load sustained for 30 minutes; write endpoints (POST) sustain 10 creates per year burst without degradation; confirmed by k6 scenario tagged M50-LOAD.
10. AC-M50-10: Module availability >= 99.99% during active by-election period (nomination close to T+30); SLO burn-rate alert fires within 5 minutes of budget breach; alert tested in chaos scenario.
11. AC-M50-11: RTO <= 30 minutes and RPO <= 5 minutes demonstrated in DR drill with Patroni fencing; no split-brain; restore logged to transparency log; drill result signed by SRE lead.
12. AC-M50-12: Tally pipeline p99 ingest-to-confirmation <= 30 seconds for by-election constituency partition under M10 peak concurrent load; regression within 10% of M10 baseline.
13. AC-M50-13: WCAG 2.2 AA: axe zero violations on all public and voter-facing surfaces; manual screen-reader test script executed and signed; confirmed in M50-A11Y test suite.
14. AC-M50-14: Public notice portal renders correctly in all 11 locales; pseudo-locale 150% expansion test produces zero truncation; ICU plural rules applied correctly for EN and SW baseline locales.
15. AC-M50-15: Log-scanner CI fixture passes with zero Tier A/B (voter PII and biometric reference) fields found in logs or traces for all by-election write paths.
16. AC-M50-16: Kafka consumer lag on eel.governance.by_election.v1 topic remains <= 60 seconds under normal load; on-call page fires within 60 seconds when lag exceeds threshold; no OOM observed in slow-consumer chaos scenario.

## Applicable edge cases

EC-DAT-01: Duplicate by-election creation request arrives from same recall event; idempotency key returns cached 201 and no second ByElection row or event is produced.
EC-DAT-02: Recall event fires twice with different polling_date in body; second call returns 422 idempotency-key-reuse; first record unchanged.
EC-DAT-03: Tally result events from M10 arrive out of order for the by-election constituency partition; buffer fills gap or routes to DLQ with reason; tally totals not applied until sequence is complete.
EC-DAT-05: Tally result arrives with votes exceeding registered voter count for the constituency; message quarantined to DLQ with EC-DAT-05 reason; partition continues; RO alerted.
EC-DAT-06: M10 sends schema v1 and v2 tally events simultaneously during a tally pipeline upgrade; both versions accepted by Karapace compat check; dual-emit processed without duplicate tallies.
EC-DAT-11: Spoilt ballots exceed valid ballots in by-election constituency; write blocked with reason; RO override creates a DisputeCase and logs override evidence.
EC-SEC-01: Kit plan HSM signature byte flipped in transit; signature verification fails on read; tamper event emitted to tamper topic; on-call paged within 30 seconds; plan not applied.
EC-SEC-03: HSM (P06) unavailable when kit plan signing is attempted; signing operation queued in degraded mode; ingest of other by-election data continues; queue drains with no gap when HSM recovers.
EC-SEC-04: KEK rotated by OpenBao transit during active by-election kit plan signing cycle; online re-wrap completes; old key readable until rotation done; full audit trail maintained in OpenBao.
EC-SEC-05: Injection attempt (SQL, shell, LDAP) sent on any by-election form input field; parameterised queries and sqlx compile-time checks prevent execution; Semgrep and ZAP scans confirm zero injection surface.
EC-SEC-16: By-election write paths emit voter register subset IDs, voter counts, or biometric references into logs or OpenTelemetry spans; log-scanner CI fixture intercepts and asserts zero Tier A/B field hits across all M50 endpoints.
EC-IDN-02: JWT expires mid-request during by-election activation POST; in-flight request completes using token valid at request start; subsequent request returns 401 with WWW-Authenticate triggering silent refresh in client.
EC-IDN-09: Dual-sign Commissioner approver has JWT revoked between first and second signature; both signatures re-validated at commit time; transaction rejected with 403; first-sign Commissioner notified to restart approval.
EC-IDN-10: SpiceDB unavailable when activation or creation write is attempted; write fails closed with 503 problem+json; class-C reads (GET /v1/by-elections) continue; circuit breaker opens after threshold.
EC-SCL-04: Kafka consumer lag on eel.governance.by_election.v1 exceeds 60 seconds; on-call page fires; backpressure applied to producer; no OOM; outbox retains events until consumer catches up.
EC-SCL-07: 10 concurrent by-elections active at election-critical load (250k check-ins, 60k RTS, 18k BVI equivalent); all M50 SLAs met at 1x load; p99 regression stays within 10% at 5x load in pre-prod simulator.
EC-LEG-01: Candidate withdraws or dies after gazette notice is published; ByElection status annotated with CANDIDATE_WITHDRAWN event; vacancy rules applied per Elections Act; nomination window reopened if below minimum candidates threshold.
EC-LEG-02: Court-ordered postponement received during active by-election; signed calendar amendment pushed via M52; kit plan refuses to open until amended polling date is confirmed; public notice portal updated within 60 seconds.
EC-LEG-09: T-equivalent auto-referral for by-election gazette fires twice due to scheduler race; single referral emitted; idempotency guard on M52 gazette call prevents duplicate notice.
EC-LEG-11: Correction to by-election declaration required after CERTIFIED status; amendment plus re-declaration requires dual custody via M51; both versions retained with proofs; tlog inclusion verified.
EC-TIM-07: Polling close time extended by signed s.14A amendment during by-election polling day; kit plan fence reopened for affected stations; forged or unsigned extension rejected with 403 and tamper event.
EC-TIM-09: Register subset freeze race at nomination close; concurrent write attempts at freeze boundary handled by OPA hard-close at transaction time; all-or-nothing freeze; no partial subset committed.
EC-CON-04: Site partition between primary and mirror during active by-election; Patroni fencing prevents split-brain; RPO <= 5 minutes confirmed; MirrorMaker lag alert fires within 60 seconds of partition.
EC-CON-11: Kafka ISR shrinks to one replica during tally event ingestion for by-election partition; producer blocks; by-election outbox accumulates in eel-outbox-rs; no tally event lost; outbox drains after ISR recovers.
EC-OPS-04: Dual-control Commissioner approver absent during by-election activation during a field emergency; time-boxed ECC remote co-sign used; co-sign event logged with ECC authorisation reference.
EC-OPS-10: PITR restore required during active by-election petition window; DR drill confirms RPO and RTO targets met; restore operation logged to transparency log; petition window deadline recalculated from restored data.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | ByElection FSM, register subset scoping, concurrency count guard | nextest | M50-UNIT- |
| property | Idempotent creation; subset voter count invariant | proptest | M50-PROP- |
| contract | M48 recall event; M51 declaration; M52 gazette | Pact | M50-PACT- |
| integration | Full by-election lifecycle; concurrent 10 instances | nextest + testcontainers | M50-INT- |
| e2e | Notice portal; admin activate; cost report | Playwright | M50-E2E- |
| load | 10 concurrent by-elections at election-critical load | k6 | M50-LOAD- |
| chaos | Kafka ISR shrink; site partition | Chaos Mesh | M50-CHAOS- |
| security | ZAP; HSM failure degraded; log scanner | ZAP + Semgrep | M50-SEC- |
| i18n | 11 locales | custom | M50-I18N- |
| a11y | axe on public surfaces | axe-core | M50-A11Y- |

Tags: `#[trace("FR-M50-007","AC-M50-01","EC-SCL-07")]`

## Runbooks and dashboards

- docs/runbooks/governance-svc-degradation.md (extend with by-election section)
- docs/runbooks/by-election-concurrent-operations.md (new)
- platform/observability/governance-svc/by-election-dashboard.json
- platform/observability/governance-svc/slo-alerts.yaml (election-critical burn rate)

## Open questions

- Maximum concurrent by-elections: inventory confirms up to 10 but legal opinion on simultaneous recall + vacancy scenarios needed.
- Whether cost ceiling is set by Treasury or the Commission Budget department; integration with M54 budget-expenditure module.
