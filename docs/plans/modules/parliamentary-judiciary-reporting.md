# M58 parliamentary-judiciary-reporting

| field | value |
|---|---|
| M-number | M58 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.1 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M58-parliamentary-judiciary-reporting.md |

## Purpose

Manages statutory reports to Parliament (select committees, speaker notifications) and judiciary handover packages (Supreme Court pre-registration, petition bundle access), including testimony workspace and version-controlled draft management.

## Surfaces and languages

| surface | technology | path under services/governance-svc/ |
|---|---|---|
| Admin reporting console | Django | django/governance-admin/ |
| Internal API | Rust (axum) | crates/reporting/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| StatutoryReport | id, report_type (parliamentary/judiciary), election_id, version, status (draft/approved/submitted/acknowledged), submitted_at, recipient | C | T1-consensus |
| Testimony | id, report_id, hearing_date, committee_ref, content_url, submitted_by | C | T1-consensus |
| JudiciaryHandover | id, declaration_id, sc_preregistration_ref, bundle_url, submitted_at, acknowledged_at | C | T1-consensus |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/reports | 3 accredited | JWT+SpiceDB | n/a | list statutory reports |
| POST /v1/reports | 3 accredited | JWT+SpiceDB | yes | create draft report |
| PUT /v1/reports/{id}/approve | 3 accredited | JWT+SpiceDB | yes | approve report (dual sign) |
| PUT /v1/reports/{id}/submit | 3 accredited | JWT+SpiceDB | yes | submit to Parliament/Judiciary |
| PUT /v1/reports/{id}/acknowledge | 3 accredited | JWT+SpiceDB | yes | record recipient acknowledgement |
| POST /v1/testimony | 3 accredited | JWT+SpiceDB | yes | submit testimony for hearing |
| POST /v1/judiciary-handover | 3 accredited | JWT+SpiceDB | yes | create judiciary handover package |
| GET /v1/judiciary-handover/{id} | 3 accredited | JWT+SpiceDB | n/a | handover package status |

OpenAPI: packages/openapi-contracts/governance/parliamentary-judiciary-reporting.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.report_submitted.v1 | emits | eel.governance.reporting.v1 | operational | no | at-least-once idempotent |
| eel.governance.judiciary_handover_submitted.v1 | emits | eel.governance.reporting.v1 | operational | no | at-least-once idempotent |
| eel.declaration.result_declared.v1 | consumes | eel.declaration.result.v1 | event-source | yes | at-least-once idempotent |
| eel.governance.court_bundle_ready.v1 | consumes | eel.governance.court.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M40 election-court-it-interface | internal | crates/reporting/ | yes | advisory; bundle reference |
| M51 result-declaration | internal | crates/reporting/ | yes | advisory; declaration triggers handover |
| M10 rts-backend | internal | crates/reporting/ | yes | advisory; tally data for reports |

## ML and AI

none

## Dependencies

- Modules: M40 (court bundle), M51 (declaration), M10 (tally)
- Platform: P01 (SpiceDB), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py

## Feature checklist

Statutory Report Management:
1. FR-M58-001: Create versioned draft statutory report in types: annual-report, post-election-report, quarterly-update, special-report. (spec FR-M58-01)
2. FR-M58-002: Track report through not-started, drafting, under-review, signed, submitted, and acknowledged states. (spec FR-M58-01)
3. FR-M58-003: Map each report section to its statutory basis (IEBC Act 2011 s.56(1), s.56(2), etc.). (spec FR-M58-02)
4. FR-M58-004: Maintain full diff view between any two report versions; version history retained permanently. (spec FR-M58-02)
5. FR-M58-005: Four-stage review workflow: drafter, Legal Counsel, CEO, Chair sign-off; each stage transition logged in P05. (spec FR-M58-02)
6. FR-M58-006: Generate HSM-signed PDF on Chair sign-off via P06; document hash appended to P05 Merkle chain. (spec FR-M58-08)
7. FR-M58-007: Record report submission with method, submitter identity, and timestamp; track to Parliament or Judiciary as applicable. (spec FR-M58-04)
8. FR-M58-008: Record recipient acknowledgement with reference number and timestamp. (spec FR-M58-04)
9. FR-M58-009: Escalate to CEO when no acknowledgement received within 5 business days of submission. (spec FR-M58-04)
10. FR-M58-010: Admin list of all reports filterable by type, election, and status; export as CSV.

Parliamentary Committee Testimony:
11. FR-M58-011: Create testimony-preparation workspace per committee appearance. (spec FR-M58-03)
12. FR-M58-012: Record committee name, appearance date, subject matter, and lead witness identity. (spec FR-M58-03)
13. FR-M58-013: Upload briefing pack PDF; track anticipated questions with draft and approved answers per question. (spec FR-M58-03)
14. FR-M58-014: Upload appearance minutes and link to testimony workspace for the same hearing. (spec FR-M58-03)
15. FR-M58-015: Admin list of all committee appearances with filter by committee and election.

Statutory Deadline Calendar:
16. FR-M58-016: Maintain statutory deadline calendar auto-calculated from election date T-0. (spec FR-M58-07)
17. FR-M58-017: Fire deadline alerts at T-60, T-30, T-14, and T-7 from each deadline; deliver to CEO and Legal Counsel. (spec FR-M58-07)
18. FR-M58-018: Display days-remaining counters with traffic-light status badges on the deadline dashboard.
19. FR-M58-019: Alert when submission deadline at T-0; escalate overdue items immediately to CEO.

Judiciary Chief Registrar Handoff:
20. FR-M58-020: Registry of formal communications with the Judiciary Chief Registrar: document type, transmission date, registrar reference, and acknowledgement status. (spec FR-M58-05)
21. FR-M58-021: Trigger judiciary handover bundle generation automatically within 60 s of consuming eel.declaration.result_declared.v1. (spec FR-M58-05)
22. FR-M58-022: Generate judiciary handover bundle with declaration, certificates, and evidence; export as HSM-signed PDF. (spec FR-M58-08)

Supreme Court Pre-Registration:
23. FR-M58-023: Track SC IT Registrar pre-registration checklist with status per item (pending, submitted, acknowledged, verified). (spec FR-M58-06)
24. FR-M58-024: Record SC IT Registrar acknowledgement and hash-verification confirmation per checklist item. (spec FR-M58-06)
25. FR-M58-025: Alert at T-14 if any pre-registration item is unacknowledged; 100% completion required by T-14. (spec FR-M58-06, NFR-M58-07)
26. FR-M58-026: Share pre-registration records with M40 Election Court IT liaison as read-only view; M40 cannot modify items. (spec FR-M58-06)

Audit and Compliance:
27. FR-M58-027: Log every draft save, version bump, sign-off, and submission in P05 with actor, timestamp, and document hash. (spec NFR-M58-06)
28. FR-M58-028: Any submitted document retrievable from M40 by hash reference; hash-path stored in P05 Merkle chain.
29. FR-M58-029: Processing under DPA 2019 s.30(1)(b) (legal obligation); testimony content upload validated to contain no individual voter PII.
30. FR-M58-030: Internal staff screens meet WCAG 2.2 AA; axe-core verification required. (spec NFR-M58-04)
31. FR-M58-031: Audit trail of all report, testimony, and handover transitions; P05 completeness verified per test cycle.
32. FR-M58-032: Deadline alert delivery confirmed 100% at each milestone by scheduler regression test with synthetic deadline set.

## Acceptance criteria

AC-M58-01: Draft versioning preserves all historical versions; diff view returns correct delta between any two versions; integration test with five version sequence.
AC-M58-02: Statutory report dual-sign requires two distinct officers; same-officer dual-sign returns 403; SpiceDB policy unit test.
AC-M58-03: Judiciary handover bundle triggered within 60 s of eel.declaration.result_declared.v1; e2e test with M51 double.
AC-M58-04: SC IT Registrar pre-registration: 100% of items acknowledged by T-14; alert fires for any unacknowledged item at T-14; scheduler test.
AC-M58-05: Report signed PDF generated by P06 HSM; document hash appended to P05 Merkle chain; hash-verification test confirms match.
AC-M58-06: Deadline alerts fire at T-60, T-30, T-14, and T-7 for 100% of tracked deadlines; scheduler regression test with synthetic IEBC Act deadline set.
AC-M58-07: No acknowledgement within 5 business days triggers CEO escalation; integration test with submission stub that withholds acknowledgement.
AC-M58-08: Testimony briefing content classified as internal; non-owner role returns 403 on briefing pack access; access-control test.
AC-M58-09: Statutory report API p99 < 1.5 s; PDF generation for 200-page documents completes < 30 s; load and timing tests.
AC-M58-10: Portal availability >= 99.5% routine; upgraded to >= 99.9% during petition window T+7 to T+180; SLO alert configured in iems-58-alerts.
AC-M58-11: Internal staff screens pass WCAG 2.2 AA; axe-core zero critical violations; manual test documented.
AC-M58-12: Every sign-off, submission, and acknowledgement event logged in P05 with actor, timestamp, and document hash; audit-log completeness test over full annual-report cycle confirms zero gaps.
AC-M58-13: Statutory reports submitted before statutory deadline for 100% of tracked obligations; compliance test seeded with IEBC Act 2011 s.56 deadlines for the 2027 election cycle.
AC-M58-14: M40 liaison read access to SC pre-registration records confirmed; M40 write attempt returns 403; access-control test.

## Applicable edge cases

EC-DAT-01: duplicate statutory report submission with same key and body returns cached 200; no duplicate forwarded to Parliament or Judiciary.
EC-DAT-02: duplicate submission with same key but different body returns 422 idempotency-key-reuse; submission rejected.
EC-DAT-06: report schema v1 and v2 coexist during format migration; both accepted; version tag preserved in document record.
EC-DAT-09: committee names and statutory references containing Unicode characters stored in NFC; display tested in EN and SW.
EC-DAT-07: partial batch failure during judiciary bundle generation; per-document error report returned; successfully generated documents committed; failed items retried.
EC-IDN-02: JWT expires mid report-draft save; in-flight save completes; next request returns 401 and triggers silent refresh.
EC-IDN-09: dual-control report sign-off approver credential revoked between first and second sign; second sign re-validates both at commit; 403 returned.
EC-IDN-10: SpiceDB unavailable; report write operations fail closed; admin reads allowed with stale-cache label; 503 returned.
EC-IDN-11: realm key rotated during active signing session; JWKS refetched on kid miss; session not invalidated before sign completes.
EC-SEC-05: injection attempted on committee reference, testimony content, and statutory basis fields; parameterised queries only; Semgrep enforced.
EC-SEC-16: report draft content and classified testimony material absent from traces and application logs; log-scanner CI fixture enforced.
EC-SEC-08: PDF uploaded from Parliament or Judiciary with malformed structure rejected; parser sandboxed; no crash; XXE disabled.
EC-SEC-03: HSM unavailable; PDF signing pauses; draft preserved; in-flight submissions queued; signing resumes on HSM recovery; no report lost.
EC-LEG-08: evidence bundle request at T+6d 23h; 24-hour statutory clock not blocked by post-election freeze; timed integration test at boundary.
EC-LEG-04: recount ordered; report version updated via compensating event; both original and amended versions retained with hash proofs.
EC-LEG-02: court-ordered election postponement; statutory deadline calendar updated via HSM-signed calendar amendment; all T-relative alerts recalculated.
EC-TIM-08: petition window T+7 midnight boundary tested at +-1 s in EAT; availability upgrade to 99.9% triggers at correct UTC timestamp.
EC-TIM-01: deadline alert timestamps stored in UTC; gazetted EAT display computed correctly; DST-safe boundary test.
EC-SCL-04: consumer lag on eel.governance.reporting.v1 exceeds 60 s; Alertmanager fires iems-58-alerts; backpressure applied; no OOM.
EC-SCL-08: Valkey eviction loses idempotency cache for report submission; PostgreSQL-durable store used as source of truth; Valkey is cache only.
EC-CON-11: Kafka ISR shrinks during report submission event; producer blocks; outbox accumulates; no event lost on ISR recovery.
EC-OPS-04: dual-control sign-off with required approver absent; time-boxed remote co-sign via ECC; event logged in audit trail.
EC-OPS-10: PITR restore during petition window; drill conducted; RPO and RTO measured; restore event logged to transparency log.
EC-HUM-08: admin console renders without truncation in EN and SW; pseudo-localisation at 150% tested.
EC-IDN-11: realm key rotated during active reporting session; JWKS refetched on kid miss; session not invalidated before in-progress sign-off completes.
- EC-DAT-03: out-of-order events on this module's consumed topics are buffered until the gap fills or routed to DLQ

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Report FSM, versioning, handover trigger | nextest | M58-UNIT- |
| property | Version chain integrity; idempotency | proptest | M58-PROP- |
| contract | M40 court bundle; M51 declaration event | Pact | M58-PACT- |
| integration | Full report lifecycle; SC pre-registration | nextest + testcontainers | M58-INT- |
| e2e | Admin report create, approve, submit | Playwright | M58-E2E- |
| security | ZAP; log scanner | ZAP + Semgrep | M58-SEC- |

Tags: `#[trace("FR-M58-007","AC-M58-03","EC-LEG-08")]`

## Runbooks and dashboards

- docs/runbooks/governance-svc-degradation.md (extend with reporting section)
- platform/observability/governance-svc/reporting-dashboard.json
- platform/observability/governance-svc/slo-alerts.yaml

## Open questions

- Draft versioning format: whether full content snapshots or structural diffs are stored; affects storage sizing.
- eCourt integration for SC pre-registration: API specification not yet available; double gates development.
