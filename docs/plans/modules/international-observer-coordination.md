# M57 international-observer-coordination

| field | value |
|---|---|
| M-number | M57 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.3 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M57-international-observer-coordination.md |

## Purpose

Manages international observer missions, pre-election briefings, observation reports, and joint statement publication, with isolated workspaces per mission to prevent inter-mission data leakage.

## Surfaces and languages

| surface | technology | path under services/governance-svc/ |
|---|---|---|
| Observer mission workspace | Elixir (Phoenix LiveView) | ex/observer_workspace/ |
| Admin mission management | Django | django/governance-admin/ |
| Public joint statement portal | Astro | astro/governance-public/ |
| Internal API | Rust (axum) | crates/intl-observer/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| ObserverMission | id, organisation_name, accreditation_ref (FK M43), lead_contact, workspace_id, status (registered/active/closed), registered_at | C | T1-consensus |
| Briefing | id, mission_id, topic, scheduled_at, materials_url, attendance_confirmed | C | T1-consensus |
| ObservationReport | id, mission_id, report_type (interim/final), submitted_at, content_url, classified | C | T1-consensus |
| JointStatement | id, participating_mission_ids, published_at, content_url, signed_by | C | T1-consensus |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/observer-missions | 3 accredited | JWT+SpiceDB | n/a | list missions |
| POST /v1/observer-missions | 3 accredited | JWT+SpiceDB | yes | register mission |
| GET /v1/observer-missions/{id}/briefings | 3 accredited | JWT+SpiceDB | n/a | mission briefings |
| POST /v1/observer-missions/{id}/briefings | 3 accredited | JWT+SpiceDB | yes | schedule briefing |
| POST /v1/observer-missions/{id}/reports | 3 accredited | JWT+SpiceDB | yes | submit observation report |
| GET /v1/observer-missions/{id}/reports | 3 accredited | JWT+SpiceDB | n/a | list reports (mission-scoped) |
| POST /v1/joint-statements | 3 accredited | JWT+SpiceDB | yes | publish joint statement |
| GET /v1/joint-statements | 1 signed-static | public | n/a | public joint statement list |

OpenAPI: packages/openapi-contracts/governance/international-observer-coordination.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.observer_report_submitted.v1 | emits | eel.governance.observer.v1 | operational | no | at-least-once idempotent |
| eel.governance.joint_statement_published.v1 | emits | eel.governance.observer.v1 | operational | no | at-least-once idempotent |
| eel.credential.observer_credential_issued.v1 | consumes | eel.credential.observer.v1 | event-source | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M15 IAB dashboard (escalation) | internal | crates/intl-observer/ | yes | advisory |
| M43 observer-accreditation | internal | crates/intl-observer/ | yes | hard dependency; mission requires accreditation |
| M11 public-portal (statement link) | internal | crates/intl-observer/ | yes | advisory |

## ML and AI

none

## Dependencies

- Modules: M15 (IAB), M43 (accreditation), M11 (portal)
- Platform: P01 (SpiceDB workspace isolation), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py, eel-web-astro

## Feature checklist

Mission Registry:
1. FR-M57-001: Register observer mission linked to M43 accreditation record; reject registration if accreditation not found. (spec FR-M57-01)
2. FR-M57-002: Track mission through pending, credentialled, deployed, and post-mission states. (spec FR-M57-01)
3. FR-M57-003: Store signed mission-agreement PDF as HSM-timestamped blob; hash stored in transparency log. (spec FR-M57-01)
4. FR-M57-004: Record organisation name, mission type (EU EOM, AU, Carter Center, IFES, Commonwealth, EAC, Other), mandate dates, Chief Observer identity, and Commission counterpart. (spec FR-M57-01)
5. FR-M57-005: Notify Commission International Affairs Officer within 5 minutes on any mission status change.
6. FR-M57-006: Admin list of all missions with status, mission type, and accreditation link; filter by status and type.

Workspace Isolation:
7. FR-M57-007: Provision isolated workspace per mission using SpiceDB namespace. (spec NFR-M57-03)
8. FR-M57-008: Prevent cross-mission data access; workspace-isolation integration test required.
9. FR-M57-009: Close and archive mission workspace after election cycle; reports retained permanently. (spec FR-M57-11 original)

Deployment Coordination:
10. FR-M57-010: Record deployment plan with observer count, county-level regions, arrival and departure dates. (spec FR-M57-02)
11. FR-M57-011: Publish deployment location overlay to M16 ECC via SSE; county-level only, no GPS or accommodation detail. (spec FR-M57-02)
12. FR-M57-012: Notify Commission International Affairs Officer on deployment-plan change within 5 minutes.
13. FR-M57-013: Version deployment plans; retain full history of changes.

Briefing Calendar:
14. FR-M57-014: Schedule briefing events (pre-election technical, election-day ops, results-transmission, other) visible to all accredited missions simultaneously. (spec FR-M57-03)
15. FR-M57-015: Record briefing agenda PDF, venue, attendance register, and minutes upload slot per briefing. (spec FR-M57-03)
16. FR-M57-016: Allow missions to submit pre-briefing questions; answers attached to briefing record after event. (spec FR-M57-03)
17. FR-M57-017: Publish briefing calendar at least T-30 before polling day; alert Commission International Affairs Officer if calendar not published in time.

Report Intake:
18. FR-M57-018: Accept Preliminary Statements, Interim Reports, and Final Reports from each mission. (spec FR-M57-04)
19. FR-M57-019: Tag each report with mission ID, report type, reporting period, language (EN, FR, SW, AR), and PDF hash. (spec FR-M57-04)
20. FR-M57-020: Issue signed report receipt token automatically within 6 hours of submission. (spec FR-M57-04)
21. FR-M57-021: Store reports permanently; cross-reference to M15 IAB quarterly report cycle. (spec FR-M57-04)
22. FR-M57-022: Mark report as classified; restrict access to Commissioner-level role only.
23. FR-M57-023: Accept reports in EN and FR; tag language; preserve original language version. (spec FR-M57-08)

Joint Statement:
24. FR-M57-024: Initiate joint statement workflow from either Commission or mission party. (spec FR-M57-05)
25. FR-M57-025: Require Commission Communications Director approval and Mission Chief Observer approval before publication; either party alone returns 403. (spec FR-M57-05)
26. FR-M57-026: Maintain version history of joint statement drafts with tracked changes. (spec FR-M57-05)
27. FR-M57-027: Log publication event with both parties' approval timestamps; publish is idempotent.

Public Portal:
28. FR-M57-028: Public page lists all accredited missions with organisation name, mission type, and mandate dates. (spec FR-M57-06)
29. FR-M57-029: Link published Final Reports as downloadable PDFs; no deployment detail or preliminary content exposed before mission marks report published. (spec FR-M57-06)

Audit and Internationalisation:
30. FR-M57-030: Log all briefing attendance, report submissions, joint-statement approvals, and deployment changes in P05 with actor, timestamp, and event type. (spec FR-M57-07)
31. FR-M57-031: Provide IAB read access to mission interaction log (not report content) via M15.
32. FR-M57-032: Portal available in EN and SW; mission briefing materials available in FR and Arabic where mission mandate requires. (spec FR-M57-08)
33. FR-M57-033: Public observer portal and mission workspace meet WCAG 2.2 AA; axe-core plus manual test required. (spec NFR-M57-04)

## Acceptance criteria

AC-M57-01: Cross-mission data access blocked at SpiceDB namespace level; mission A JWT cannot read mission B reports; integration test with two active missions.
AC-M57-02: Mission registration rejected with 422 if no valid M43 accreditation record exists for the organisation; Pact contract test with M43 double.
AC-M57-03: Joint statement published idempotently; eel.governance.joint_statement_published.v1 emitted exactly once even on retry; idempotency test.
AC-M57-04: Both Commission and mission party approval required before publication; single-party approve returns 403; state-machine unit test.
AC-M57-05: Report receipt acknowledgement issued within 6 hours for 100% of submissions; scheduler test with synthetic submission set covering all mission types.
AC-M57-06: Deployment overlay delivered to M16 ECC SSE stream within p99 < 60 s on plan update; integration test measured over 50 deployment updates.
AC-M57-07: Briefing calendar published at least T-30 before polling day; deadline alert fires if calendar unpublished by that date; scheduler test.
AC-M57-08: Mission workspace API p99 < 1.5 s; briefing calendar API p99 < 1 s; k6 load test at 450 concurrent mission users.
AC-M57-09: Mission portal availability >= 99.5% T-365 to T+90; SLO alert configured in iems-57-alerts.
AC-M57-10: Classified reports inaccessible to Commission staff without Commissioner-level role; SpiceDB role test confirms 403 for lower roles.
AC-M57-11: Public portal and mission workspace pass WCAG 2.2 AA in EN and SW; axe-core zero critical violations; manual test documented.
AC-M57-12: All mission events logged in P05 with actor, timestamp, and event type; audit-log completeness test over a full mission lifecycle confirms zero gaps.
AC-M57-13: Deployment data published to M16 contains no GPS coordinates, accommodation details, or report content; field-presence test on M16 SSE payload.
AC-M57-14: Mission portal supports EN, SW, FR, and Arabic (4 languages); locale switch test for all four languages on mission workspace and public portal screens.

## Applicable edge cases

EC-DAT-01: duplicate observation report submission with same key and body returns cached 200; no second ingestion triggered.
EC-DAT-02: duplicate submission with same key but different body returns 422 idempotency-key-reuse; report rejected.
EC-DAT-06: report schema v1 and v2 coexist during migration; both accepted; compat check via Karapace verified.
EC-DAT-09: observer and mission names containing Arabic and French diacritics stored in NFC, collated with ICU; locale snapshot tested across 4 active languages.
EC-DAT-14: oversized mission-agreement PDF exceeding 1 MB outbox limit rejected from outbox; redirected to MinIO blob with hash reference.
EC-IDN-02: JWT expires mid report-upload; in-flight request completes; next request returns 401 and triggers silent refresh.
EC-IDN-09: joint-statement dual-approver credential revoked between first and second approval; second approval re-validates both at commit; 403 returned.
EC-IDN-10: SpiceDB unavailable; workspace read and write operations fail closed; 503 returned with problem+json.
EC-IDN-11: realm key rotated while mission LiveView session is active; JWKS refetched on kid miss; session not dropped.
EC-SEC-05: injection attempted on mission-name, joint-statement draft text, and briefing-question fields; parameterised queries only; Semgrep enforced.
EC-SEC-15: LiveView session hijack attempt on observer workspace; CSRF token validated; session binding enforced; re-auth on step-up triggered.
EC-SEC-16: classified report content and mission personal data (Chief Observer email) absent from traces and application logs; log-scanner CI fixture enforced.
EC-SEC-06: SSRF via report URL or briefing materials URL field; allowlist enforced; private IP ranges blocked.
EC-SEC-08: joint-statement PDF upload with malformed PDF structure rejected; parser sandboxed; no crash; XXE disabled.
EC-SEC-12: public observer portal snapshot cache poisoned; cache key includes signature version; client verifies before serving.
EC-CON-05: flapping M43 accreditation service; circuit breaker opens after retry budget; mission registration fails closed with clear error.
EC-CON-11: Kafka ISR shrinks during joint-statement publish event; producer blocks; outbox accumulates; no event loss on ISR recovery.
EC-SCL-04: consumer lag on eel.governance.observer.v1 exceeds 60 s; Alertmanager fires iems-57-alerts; backpressure applied; no OOM.
EC-SCL-01: public observer portal serves 5M visitors per hour at >= 95% CDN edge; origin p99 < 500 ms; k6 and Gatling 5-region test.
EC-TIM-06: mission report submitted with timestamp earlier than briefing event; audit skew anomaly logged; report accepted and not rejected.
EC-LEG-02: court-ordered postponement changes election date; mission mandate dates updated via HSM-signed calendar amendment; workspace notified within 60 s.
EC-HUM-08: mission workspace and public portal render without truncation in all 11 locales; pseudo-localisation at 150% tested.
EC-HUM-09: public observer portal uses symbol and pattern alongside colour for mission-status indicators; ΔE contrast registry check passes.
EC-OPS-07: mission coordinator credential reassigned due to staff no-show; old credential revoked; new credential issued exactly once; old sessions invalidated.
EC-SCL-03: hot partition on observer topic during simultaneous report submissions from all missions; stream-id spread across partitions; lag alert fires; HPA scales consumers.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Mission FSM, workspace isolation, report access control | nextest | M57-UNIT- |
| property | Cross-mission isolation invariant | proptest | M57-PROP- |
| contract | M43 accreditation; M11 statement link | Pact | M57-PACT- |
| integration | Full mission lifecycle; workspace isolation | nextest + testcontainers | M57-INT- |
| e2e | Admin mission registration; report submission; joint statement | Playwright | M57-E2E- |
| security | ZAP; session hijack; log scanner | ZAP + Semgrep | M57-SEC- |
| i18n | 11 locales | custom | M57-I18N- |
| a11y | axe on public and workspace surfaces | axe-core | M57-A11Y- |

Tags: `#[trace("FR-M57-003","AC-M57-01","EC-IDN-10")]`

## Runbooks and dashboards

- docs/runbooks/governance-svc-degradation.md (extend with observer section)
- platform/observability/governance-svc/observer-dashboard.json
- platform/observability/governance-svc/slo-alerts.yaml

## Open questions

- Workspace isolation strategy: SpiceDB namespace per mission vs. row-level policies; decision affects schema design.
- Whether classified reports require encryption at rest beyond standard Tier B handling.
