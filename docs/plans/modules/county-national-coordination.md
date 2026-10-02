# M56 county-national-coordination

| field | value |
|---|---|
| M-number | M56 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.3 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M56-county-national-coordination.md |

## Purpose

Manages MoUs, county-level dataset approvals, ministerial coordination, and cross-tier incident escalation between county the Commission offices and the national Commission, with SAML brokering for heterogeneous county government identity providers.

## Surfaces and languages

| surface | technology | path under services/governance-svc/ |
|---|---|---|
| County coordination portal | Astro | astro/governance-public/ (county subdomain) |
| Admin coordination console | Django | django/governance-admin/ |
| Internal API | Rust (axum) | crates/county-coord/ |
| SAML broker | Elixir (Phoenix) | ex/saml_broker/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| MoU | id, county_code, ministry_code, effective_date, expiry_date, content_url, status (draft/signed/active/expired), signed_at | C | T1-consensus |
| CountyDataset | id, county_code, dataset_type, source_system, approved_by, approval_date, dataset_url | C | T1-consensus |
| MinistryApproval | id, mou_id, ministry_ref, approved_by, approved_at | C | T1-audit |
| CoordinationIncident | id, county_code, incident_type, severity, reported_at, resolved_at, ecc_alert_ref | C | T1-consensus |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/coordination/mous | 3 accredited | JWT+SpiceDB | n/a | list MoUs |
| POST /v1/coordination/mous | 3 accredited | JWT+SpiceDB | yes | create MoU |
| PUT /v1/coordination/mous/{id}/sign | 3 accredited | JWT+SpiceDB | yes | dual-sign MoU |
| GET /v1/coordination/datasets | 3 accredited | JWT+SpiceDB | n/a | list county datasets |
| POST /v1/coordination/datasets | 3 accredited | JWT+SpiceDB | yes | submit county dataset |
| POST /v1/coordination/incidents | 3 accredited | JWT+SpiceDB | yes | report coordination incident |
| PUT /v1/coordination/incidents/{id}/resolve | 3 accredited | JWT+SpiceDB | yes | resolve incident |
| GET /v1/coordination/incidents | 3 accredited | JWT+SpiceDB | n/a | list incidents |
| POST /v1/auth/saml/acs | internal gRPC | SVID mTLS | yes | SAML ACS endpoint for county IdPs |

OpenAPI: packages/openapi-contracts/governance/county-national-coordination.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.coordination_incident_raised.v1 | emits | eel.governance.coordination.v1 | operational | no | at-least-once idempotent |
| eel.governance.mou_signed.v1 | emits | eel.governance.coordination.v1 | operational | no | at-least-once idempotent |
| eel.ecc.alert_raised.v1 | consumes | eel.ecc.alert.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| IDMS SAML (county IdPs) | idms-saml | integrations/ (tier B, SAML) | yes | circuit breaker; SAML auth fails closed |
| M16 election-command-centre | internal | crates/county-coord/ | yes | advisory; incident escalated to ECC |
| M18 boundary-transparency | internal | crates/county-coord/ | yes | advisory; boundary dataset reference |
| M33 personnel-dispatch | internal | crates/county-coord/ | yes | advisory; personnel incident link |

## ML and AI

none

## Dependencies

- Modules: M16 (ECC escalation), M18 (boundary reference), M33 (personnel)
- Platform: P01 (SpiceDB), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py, eel-web-astro

## Feature checklist

MoU Management:
1. FR-M56-001: Create MoU with county or ministry party, recording party type, party code, and party name. (spec FR-M56-01)
2. FR-M56-002: Dual-sign MoU with two distinct officer signatures; SpiceDB policy enforces separation of duties. (spec FR-M56-01)
3. FR-M56-003: Track MoU through draft, review, signed, active, and expired states. (spec FR-M56-01)
4. FR-M56-004: Generate expiry alerts at T-90 and T-30 days before MoU expiry; 100% delivery rate. (spec FR-M56-01)
5. FR-M56-005: Trigger MoU renewal workflow automatically when expiry alert fires.
6. FR-M56-006: Store signed MoU as HSM-timestamped PDF blob; hash stored in transparency log. (spec FR-M56-01)
7. FR-M56-007: Version-control MoU drafts; diff view between any two versions.
8. FR-M56-008: Ministry approval tracking for cross-ministry MoUs; link approval to originating MoU. (spec FR-M56-04)

County Dataset Management:
9. FR-M56-009: Accept county dataset uploads in five types: population, school-facility, health-facility, road-access, security-history. (spec FR-M56-02)
10. FR-M56-010: Validate each county dataset against published JSON Schema v7 before acceptance; reject non-conforming payloads. (spec FR-M56-02)
11. FR-M56-011: Return field-level validation errors to the uploader on schema failure, referencing the failing field and rule.
12. FR-M56-012: Tag every data point with provenance: source county, uploader, timestamp, and schema version. (spec FR-M56-03)
13. FR-M56-013: Export merged polling-station siting-layer dataset to M18 Boundary Transparency within 60 s of dataset approval. (spec FR-M56-03)
14. FR-M56-014: Scope county dataset access to the submitting county; cross-county request returns 403.
15. FR-M56-015: Capture DPA 2019 s.30(1)(d) lawful-basis code and purpose code on every dataset upload; absence returns 422. (spec FR-M56-06)
16. FR-M56-016: Set retention-until date on every dataset record per the retention schedule; field cannot be blank.

Ministerial Approval Tracking:
17. FR-M56-017: Create structured ministerial approval request with statutory basis, deadline, and escalation path. (spec FR-M56-04)
18. FR-M56-018: Track approval through pending, approved, rejected, and overdue states.
19. FR-M56-019: Alert Director of Operations when an approval passes its deadline. (spec FR-M56-04)
20. FR-M56-020: Record ministry response with timestamp and responding officer identity. (spec FR-M56-04)

Incident Coordination:
21. FR-M56-021: Report cross-jurisdiction incidents in five types: road-closure, health-emergency, security-event, infrastructure, other. (spec FR-M56-05)
22. FR-M56-022: Link incidents to one or more polling-station codes and a county code.
23. FR-M56-023: Emit eel.governance.coordination_incident_raised.v1 to M16 ECC; p99 delivery < 30 s. (spec FR-M56-05)
24. FR-M56-024: Track incident through open, assigned, and resolved states.
25. FR-M56-025: Require sign-off from both a Commission officer and a county officer to close an incident. (spec FR-M56-05)

SAML Brokering and Authentication:
26. FR-M56-026: SAML brokering for heterogeneous county-government IdPs via SAML ACS endpoint. (spec NFR-M56-03)
27. FR-M56-027: Fail closed on SAML IdP unavailability; no partial authentication granted; circuit breaker logged.
28. FR-M56-028: Support TOTP-based authentication for Commission-issued county credentials.

Coordination Calendar and Portal:
29. FR-M56-029: Maintain coordination calendar with attendee lists, minutes-upload slots, and action-item tracker. (spec FR-M56-07)
30. FR-M56-030: County coordination portal available under county subdomain with county-scoped RBAC.
31. FR-M56-031: Admin console lists all incidents, MoUs, and datasets with filter by county, type, and status.

DPA Audit and Internationalisation:
32. FR-M56-032: Log every inbound and outbound data exchange in P05 with actor, timestamp, event type, and DPA purpose code. (spec FR-M56-06)
33. FR-M56-033: Produce DPA audit report for any queried time window via `/v1/dpa/audit`; row count verifiable against P05 log.
34. FR-M56-034: County portal and admin console available in EN and SW; all upload templates provided in EN and SW with field-level guidance. (spec FR-M56-08)
35. FR-M56-035: County-facing portal meets WCAG 2.2 AA; verified by axe-core and manual test in EN and SW. (spec NFR-M56-04)

## Acceptance criteria

AC-M56-01: SAML ACS fails closed on IdP unavailability; no partial auth granted; integration test with IdP stub returning 503.
AC-M56-02: High-severity incident emits eel.governance.coordination_incident_raised.v1 visible in M16 ECC within p99 < 30 s; measured over 50 runs.
AC-M56-03: MoU dual-sign requires two distinct officers; same-officer dual-sign returns 403; SpiceDB policy unit test.
AC-M56-04: County dataset access scoped to submitting county; cross-county request returns 403; access-control integration test.
AC-M56-05: Dataset upload rejected with field-level errors when payload fails JSON Schema v7 validation; unit test per dataset type.
AC-M56-06: MoU expiry alerts delivered for 100% of expiring MoUs at T-90 and T-30; scheduler test with synthetic MoU set covering all 47 counties.
AC-M56-07: Every dataset upload record carries a DPA lawful-basis code and purpose code; absence of either returns 422; unit test.
AC-M56-08: County coordination portal API p99 < 1.5 s at 500 concurrent county users; k6 load test at synthetic 500-user concurrency.
AC-M56-09: Portal availability >= 99.5% over T-180 to T+30 window; SLO alert configured in Alertmanager ruleset iems-56-alerts.
AC-M56-10: Incident resolution requires both Commission and county officer sign-off; single-party close returns 403; integration test.
AC-M56-11: County portal and upload wizard pass WCAG 2.2 AA in EN and SW; axe-core zero critical violations; manual test documented.
AC-M56-12: All inbound and outbound data exchanges logged in P05 with actor, timestamp, and DPA purpose code; audit-log completeness test over a full county-upload cycle confirms zero gaps.
AC-M56-13: Merged siting-layer export reaches M18 within 60 s of dataset approval; integration test with M18 double.
AC-M56-14: Incident routing to M16 ECC p99 < 30 s sustained under 47 concurrent county incident submissions; k6 scenario test.

## Applicable edge cases

EC-DAT-01: duplicate county dataset upload with same key and body returns cached 200; no second validation run triggered.
EC-DAT-02: duplicate county dataset upload with same key but different body returns 422 idempotency-key-reuse; upload rejected.
EC-DAT-03: out-of-order incident events in the coordination stream buffered until gap filled or routed to DLQ; replay correctness tested.
EC-DAT-06: county dataset schema v1 and v2 coexist during migration; both accepted; dual-emit verified in Karapace compat test.
EC-DAT-09: county and sub-location names containing Unicode grapheme clusters stored in NFC, collated with ICU; display tested across 11 locales.
EC-DAT-14: county dataset upload exceeding 1 MB outbox limit rejected from outbox; blob redirected to MinIO with hash reference.
EC-IDN-02: JWT expires mid county-upload request; in-flight request completes; next request returns 401 and triggers silent refresh.
EC-IDN-09: MoU dual-control approver credential revoked between first and second sign; second sign re-validates both at commit; 403 returned.
EC-IDN-10: SpiceDB unavailable; write operations fail closed; class-C reads allowed with stale-cache label; 503 returned with problem+json.
EC-IDN-11: realm key rotated while county SAML session is active; JWKS refetched on kid miss; session not invalidated.
EC-SEC-05: injection attempted on MoU content fields, dataset URL, and incident description; parameterised queries only; ZAP and Semgrep verified.
EC-SEC-06: SSRF attempted via county dataset URL field; allowlist enforced; private IP ranges blocked; rebinding guard active.
EC-SEC-08: SAML response containing XXE payload rejected; DTD disabled in SAML parser; billion-laughs fuzz verified.
EC-SEC-16: sensitive county demographic data fields (population counts, facility GPS coordinates) absent from traces and application logs; log-scanner fixture enforced in CI.
EC-CON-05: flapping county SAML IdP triggers exponential backoff with jitter; fail closed after retry budget exhausted; no token leaked.
EC-CON-08: DNS poisoned for county SAML IdP endpoint; DNSSEC validation fails; hardcoded fallback endpoint used; circuit opens.
EC-CON-11: Kafka ISR shrinks during incident event publication; producer blocks; incident outbox accumulates; no event loss; drains on ISR recovery.
EC-SCL-04: Kafka consumer lag on eel.governance.coordination.v1 exceeds 60 s; Alertmanager fires iems-56-alerts; backpressure applied; no OOM.
EC-SCL-03: hot partition on coordination topic when all 47 counties submit datasets simultaneously; stream-id spread across partitions; lag alert fires; HPA scales consumers.
EC-TIM-06: incident event timestamp earlier than causal event accepted; audit skew anomaly logged; stream ordering unaffected.
EC-LEG-03: s.44A invoked; incident coordination log records the manual-ID override event with dual-officer sign-off and routes alert to ECC.
EC-HUM-08: county portal and upload wizard render correctly in all 11 languages with no truncation; pseudo-localisation at 150% tested.
EC-HUM-09: county portal incident map uses symbol and pattern alongside colour for severity; ΔE contrast registry check passes.
EC-OPS-07: county SAML coordinator credential reassigned due to staff no-show; old credential revoked; new credential issued with exactly-once guarantee.
EC-SCL-08: Valkey eviction loses idempotency cache for dataset upload; PostgreSQL-durable store used as authoritative source; Valkey is cache only.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | MoU FSM, incident severity routing, SAML ACS validation | nextest | M56-UNIT- |
| property | County scoping invariant; dual-sign SoD | proptest | M56-PROP- |
| contract | SAML IdP double; M16 ECC alert | Pact | M56-PACT- |
| integration | Full MoU lifecycle; SAML auth; incident escalation | nextest + testcontainers | M56-INT- |
| e2e | County portal; admin MoU and incident flows | Playwright | M56-E2E- |
| security | ZAP; SAML XXE; log scanner | ZAP + Semgrep | M56-SEC- |
| i18n | 11 locales | custom | M56-I18N- |
| a11y | axe on county portal | axe-core | M56-A11Y- |

Tags: `#[trace("FR-M56-006","AC-M56-01","EC-SEC-08")]`

## Runbooks and dashboards

- docs/runbooks/governance-svc-degradation.md (extend with coordination section)
- docs/runbooks/county-saml-broker-failure.md (new)
- platform/observability/governance-svc/coordination-dashboard.json
- platform/observability/governance-svc/slo-alerts.yaml

## Open questions

- Heterogeneous county IdP metadata: not all 47 counties operate the same SAML provider; onboarding schedule and metadata update process to be defined.
- County subdomain routing strategy (DNS vs. path-prefix) for the county portal.
