# M52 election-calendar-gazette

| field | value |
|---|---|
| M-number | M52 |
| Home service | portal-svc (ADR-0020) |
| Owning work package | WP-5.7 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M52-election-calendar-gazette.md |

## Purpose

Publishes and manages the statutory election calendar and gazette notices, enforcing a rule engine for legal deadlines, submitting notices to the Government Printer, and enabling public subscription to calendar events.

## Surfaces and languages

| surface | technology | path under services/portal-svc/ |
|---|---|---|
| Public calendar and gazette portal | Astro | astro/gazette/ |
| Admin gazette management | Django | django/portal-admin/ |
| Internal API | Rust (axum) | crates/gazette/ |
| Statutory rule engine | Rust | crates/gazette/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| CalendarEvent | id, election_id, event_type, statutory_basis, scheduled_date, status (draft/approved/published/amended), published_at | C | T1-consensus |
| GazetteNotice | id, calendar_event_id, notice_number, content_html, content_pdf_url, gov_printer_ref, legal_approval_at, signed_at | C | T1-consensus |
| StatutoryRule | id, rule_code, description, days_before_election, mandatory | C | T1-consensus |
| Subscription | id, subscriber_email_hash, event_types, created_at | D | T2 |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/calendar/events | 1 signed-static | public | n/a | public calendar feed (JSON + iCal) |
| GET /v1/calendar/events/{id} | 1 signed-static | public | n/a | event detail with gazette link |
| GET /v1/gazette/notices | 1 signed-static | public | n/a | list published notices |
| GET /v1/gazette/notices/{id} | 1 signed-static | public | n/a | notice content and PDF |
| POST /v1/gazette/notices | 3 accredited | JWT+SpiceDB | yes | create draft notice |
| PUT /v1/gazette/notices/{id}/approve | 3 accredited | JWT+SpiceDB | yes | legal approval |
| PUT /v1/gazette/notices/{id}/publish | 3 accredited | JWT+SpiceDB | yes | sign and publish; submit to Gov Printer |
| POST /v1/calendar/subscriptions | 2 live-per-user | JWT | yes | subscribe to calendar events |

OpenAPI: packages/openapi-contracts/portal/election-calendar-gazette.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.portal.gazette_notice_published.v1 | emits | eel.portal.gazette.v1 | operational | no | at-least-once idempotent |
| eel.portal.calendar_event_amended.v1 | emits | eel.portal.gazette.v1 | operational | no | at-least-once idempotent |
| eel.declaration.result_declared.v1 | consumes | eel.declaration.result.v1 | event-source | yes | at-least-once idempotent |
| eel.governance.referendum_activated.v1 | consumes | eel.governance.referendum.v1 | operational | no | at-least-once idempotent |
| eel.governance.by_election_created.v1 | consumes | eel.governance.by_election.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| Government Printer API | gov-printer | integrations/gov-printer/ | yes | circuit breaker; notice queued for retry |
| M51 result-declaration | internal | crates/gazette/ | yes | hard dependency for certificate gazette |
| M49 referendum-mechanics | internal | crates/gazette/ | yes | advisory; referendum calendar |
| M50 by-election-operations | internal | crates/gazette/ | yes | advisory; by-election notice |
| M18 boundary-transparency | internal | crates/gazette/ | yes | advisory; boundary change notices |

## ML and AI

none

## Dependencies

- Modules: M51 (declaration gazette), M49 (referendum), M50 (by-election), M18 (boundary)
- Platform: P04 (Kafka outbox), P05 (transparency log for signed notices)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-astro, eel-web-py

## Feature checklist

Election event management:
1. FR-M52-001: Create election event with type (GENERAL_ELECTION, BY_ELECTION, REFERENDUM, PARTY_PRIMARY), polling date T-0, scope, and scope code; validate type against Elections Act classification rules.
2. FR-M52-002: Auto-generate full statutory notice calendar on election event creation, populating all required milestone dates within 5 minutes.
3. FR-M52-003: Statutory rule engine computes notice publication date, earliest and latest permissible window, and minimum-notice-days for each notice type from a versioned rule set; Elections Act statutory basis cited per rule.
4. FR-M52-004: Rule set versioned with effective-from date; historical elections use the rule set active at the time; rule changes require legal-expert sign-off before activation; version stored on each notice record.
5. FR-M52-005: Prerequisite-chain enforcement: block notice submission when its trigger event is incomplete (e.g. nomination-close notice must precede campaign-open notice); block returns specific statutory citation.
6. FR-M52-006: Statutory compliance check before submission: publication date within window, prerequisites complete, approvals done; submission blocked with statutory violation code when any check fails.
7. FR-M52-007: Commissioner override of blocked submission requires reason logged; override itself recorded in immutable audit log.

Notice authoring and approval workflow:
8. FR-M52-008: Draft gazette notice in EN and SW using statutory templates; statutory basis citation (e.g. Elections Act s.22(1)) displayed on every notice record.
9. FR-M52-009: Legal Officer approval step enforced by FSM; notice cannot advance to SIGNED without LEGAL_REVIEW approval.
10. FR-M52-010: Commissioner HSM signature step for notices flagged requires_commissioner_signature; signature produced via P06 HSM; key ID and signature stored with notice record.
11. FR-M52-011: Submit signed notice package to Government Printer API; receive and store gazette volume, issue number, and publication date on confirmation.
12. FR-M52-012: Government Printer circuit breaker with exponential-backoff retry up to 24 hours on API failure; notice held in outbox and retried without data loss.
13. FR-M52-013: Manual upload fallback: system packages notice PDF for email submission to Government Printer when API unavailable; human-tracked confirmation recorded.
14. FR-M52-014: Amend published notice via compensating event with mandatory reason; original notice record immutable; amendment linked to original by notice-id; new notice number assigned per Government Printer rules.
15. FR-M52-015: Sign published notice snapshot with HSM-anchored key; signature included in JSON response and PDF footer; verifiable by external parties without system access.
16. FR-M52-016: Immutable audit log of every notice FSM transition (draft, legal-review, approved, signed, submitted, published, overdue) with actor, timestamp, and reason; retained 10 years post-election; no soft-delete.

Calendar and public access:
17. FR-M52-017: Public calendar served in JSON and iCal (RFC 5545) formats; each event includes statutory basis, gazette volume and issue reference, and countdown to next action.
18. FR-M52-018: CDN-edge-served calendar with signed snapshots; signed-static tier; p95 200 ms / p99 500 ms at 5M visitors per hour; origin 99.5% availability.
19. FR-M52-019: Notice content available as HTML and version-stamped PDF with statutory formatting; PDF generation idempotent on same notice version.
20. FR-M52-020: RSS feed for gazette notices; items link to notice HTML and PDF.
21. FR-M52-021: Historical archive of all past election gazette notices retained; searchable by election year, notice type, and constituency.
22. FR-M52-022: Public subscription to calendar events via hashed subscriber email; subscriber email never stored raw; no PII in logs, traces, or Kafka event payloads.
23. FR-M52-023: Gantt-style election calendar timeline UI colour-coded by status (upcoming, pending-action, overdue, published); exportable as PDF.
24. FR-M52-024: Admin notice list with filter by election, status, notice type, and notice number; countdown widgets per pending notice; escalation history visible.

Cross-notification and event integration:
25. FR-M52-025: Emit eel.portal.gazette_notice_published.v1 on gazette confirmation; cross-notify M04, M12, M22, M55, M20 and other subscribers within 60 s; at-least-once idempotent delivery.
26. FR-M52-026: Consume eel.declaration.result_declared.v1 from M51; auto-create draft result gazette notice; idempotent on replay (duplicate events produce no second draft).
27. FR-M52-027: Consume eel.governance.referendum_activated.v1 from M49; auto-create referendum gazette notices including question text, voting date, and county-threshold rules.
28. FR-M52-028: Consume eel.governance.by_election_created.v1 from M50; auto-generate abbreviated statutory calendar for the single-constituency event.
29. FR-M52-029: Consume boundary-change events from M18; auto-create boundary gazette notice draft for Calendar Officer review.

Alerting, escalation, and observability:
30. FR-M52-030: Generate alerts at T-14, T-7, T-3, and T-1 days before each statutory deadline; escalate to Commissioner at T-1 if notice not submitted; one-click sign-and-submit from escalation alert.
31. FR-M52-031: Escalation alert for Government Printer non-confirmation within 48 hours of submission; alert routed to Commissioner.
32. FR-M52-032: OpenTelemetry traces exported to central collector; Grafana dashboard eel-calendar-gazette with per-notice SLO burn alerts; Alertmanager ruleset eel-calendar-gazette-alerts; dashboard and ruleset gate FAT-07.

Internationalisation and accessibility:
33. FR-M52-033: i18n of all calendar event labels and notice templates in 11 languages; no hardcoded strings; 11 locales + 150% pseudo-locale expansion pass.
34. FR-M52-034: WCAG 2.2 AA compliance on public calendar portal and IEBC notice workspace; axe-core zero violations; manual screen-reader script signed off.

## Acceptance criteria

1. AC-M52-01: Statutory rule engine computes correct milestone dates for all boundary cases (election_date plus or minus rule offset, including zero-day and negative offsets); property tests cover >= 5 notice types with 1,000 generated election-date inputs; zero miscalculations.
2. AC-M52-02: Full statutory notice calendar for a general election (47 notices) generated within 5 minutes of election event creation; measured in integration test with real DB.
3. AC-M52-03: Commissioner HSM signature accepted; Government Printer double returns 200; notice status transitions to PUBLISHED; contract test against Gov Printer double passes.
4. AC-M52-04: Gov Printer API unavailable triggers circuit breaker within 3 failed attempts; notice remains in outbox; retry succeeds on reconnect; no notice data lost; chaos test with Gov Printer double offline for 10 minutes passes.
5. AC-M52-05: Gov Printer retry queue exhausts exponential-backoff within 24 hours; escalation alert sent to Commissioner if publication not confirmed within 48 hours of submission.
6. AC-M52-06: Cross-notification events reach all subscribing modules within 60 s of gazette confirmation; measured in integration test with M04, M12, M22, M55 consumer doubles.
7. AC-M52-07: eel.declaration.result_declared.v1 replayed three times produces exactly one draft gazette notice; idempotency key prevents duplicates; integration test passes.
8. AC-M52-08: Amending a PUBLISHED notice creates a compensating event; original notice record unchanged after amendment; kani/proptest append-only property passes; amended notice linked to original by id.
9. AC-M52-09: Statutory compliance check blocks submission with a specific statutory citation (e.g. Elections Act s.11(3)) when publication date is outside the permissible window; test covers earliest and latest boundary by one second.
10. AC-M52-10: Public calendar p95 <= 200 ms / p99 <= 500 ms at 5M visitors per hour in k6 load test; origin availability >= 99.5%; p99 regression <= 10% at 5x load.
11. AC-M52-11: 11 locale snapshot tests pass; iCal feed dates are locale-correct; no string truncation at 150% pseudo-locale expansion; font-fallback renders all 11 scripts.
12. AC-M52-12: axe-core reports zero WCAG 2.2 AA violations on public portal and admin workspace; manual screen-reader task-completion script signed by QA; Playwright a11y test tagged M52-A11Y passes.
13. AC-M52-13: Audit log records every FSM transition; no DELETE or UPDATE on log rows succeeds (DB constraint enforced); 10-year retention rule verified by schema constraint; log-scanner CI detects no PII.
14. AC-M52-14: Subscriber email stored only as SHA-256 hash; raw email absent from Postgres rows, Kafka payloads, OTel traces, and application logs; log-scanner CI fixture passes.
15. AC-M52-15: Notice PDF is version-stamped, HSM-signed, and verifiable against public key without system access; PDF regeneration for same notice version is byte-for-byte idempotent.

## Applicable edge cases

Data integrity:
- EC-DAT-01: duplicate POST /v1/gazette/notices with same idempotency key returns cached 201 response; notice created exactly once; auto-generated test per write endpoint.
- EC-DAT-02: duplicate POST with same idempotency key but different body returns 422 idempotency-key-reuse; original notice body preserved unchanged.
- EC-DAT-03: out-of-order eel.declaration.result_declared.v1 events buffered until predecessor committed; no phantom or duplicate gazette notice created.
- EC-DAT-06: GazetteNotice schema v1 and v2 coexist in Kafka topic during migration; consumer accepts both; dual-emit path verified with Karapace compat check.
- EC-DAT-09: notice text containing Unicode candidate names (Swahili diacritics, Arabic script in diaspora scope) stored as NFC; retrieved and displayed without corruption; 11-locale snapshot covers all scripts.
- EC-DAT-14: oversized gazette notice body (> 1 MB) rejected by outbox; content offloaded to MinIO with SHA-256 hash reference; notice record carries the hash only.

Security:
- EC-SEC-05: SQL injection and XSS attempted on all notice text input fields; parameterised queries only; ZAP DAST scan and Semgrep SAST pass; fuzz corpus committed.
- EC-SEC-12: CDN cache poisoning of gazette signed snapshots; cache key includes signature version; client SDK verifies HSM signature before rendering notice; Varnish injection test passes.
- EC-SEC-16: subscriber email hashed before any storage or emission; raw email absent from Postgres rows, Kafka payloads, OTel traces, and application logs; log-scanner CI regex fixture passes.
- EC-SEC-03: HSM unavailable during Commissioner signature step; signature queue accumulates in outbox; degraded-mode alert sent; no notice submitted without a valid HSM signature; test with SoftHSM killed.
- EC-SEC-09: rate-limit evasion on public calendar endpoint; limits keyed by subject, device, and /64 prefix; priority shedding under load; Nuclei test passes.

Identity:
- EC-IDN-02: JWT expires between Legal Officer clicking approve and server commit; in-flight request completes; subsequent request returns 401 with silent-refresh opportunity; Keycloak 5 s TTL test.
- EC-IDN-09: dual-control notice approval where second approver credential is revoked between first and second sign-off; both identities re-validated at commit; revocation-between-steps integration test passes.
- EC-IDN-10: SpiceDB down during notice approval write; write fails closed with 503 problem+json; CDN-served public calendar (class-C read) remains available; PodChaos test passes.
- EC-IDN-03: SPIFFE SVID rotates during Gov Printer batch submission; old SVID valid until expiry; submission completes without interruption; SPIRE 30 s rotation test passes.

Legal and statutory:
- EC-LEG-02: court-ordered postponement triggers HSM-signed calendar amendment; all downstream subscribers receive amended milestone dates via eel.portal.calendar_event_amended.v1; portal notice published; kits instructed via bundle; e2e test passes.
- EC-LEG-04: court-ordered recount requires result gazette amendment; original PUBLISHED notice immutable; compensating notice created with court order reference and new notice number; append-only kani property passes.
- EC-LEG-01: candidate withdraws after nomination gazette notice is published; original notice annotated with withdrawal reference; M12 vacancy rules applied; M52 publishes amended notice; rules-unit and TLA+ annotated-candidate tests pass.
- EC-LEG-09: T-60 auto-referral scheduler fires twice due to duplicate Kafka delivery; only one referral created; idempotent scheduler double-fire test passes.
- EC-LEG-05: presidential run-off required by rules engine (no candidate reaches 50%+1); run-off calendar auto-generated with abbreviated statutory notice periods; TLA+ run-off property passes.

Time and timezone:
- EC-TIM-04: all calendar deadline dates stored in UTC; displayed in gazetted Africa/Nairobi time; DST boundary day (if Kenya ever adopts DST) handled correctly; tz snapshot test plus mock-DST kit test passes.
- EC-TIM-07: statutory deadline extended by gazette amendment (e.g. nomination window extended by 2 days); HSM-signed amendment propagated to all subscribers; forged unsigned extension rejected by signature check.
- EC-TIM-08: petition window T+28d midnight boundary (Elections Act s.75(1)); calendar event boundary test precise to the second; both inclusive-end and exclusive-end variants in boundary table; boundary-table test +-1 s passes.

Scale and throughput:
- EC-SCL-01: 5M visitors per hour on public calendar portal; >= 95% of requests served from CDN edge; origin p99 <= 500 ms; k6 + Gatling 5-region test passes.
- EC-SCL-02: thundering herd on CDN snapshot refresh after gazette publication; SDK jitter applied; ETag 304 returned for unchanged calendar version; 100k verifiers simulator passes.
- EC-SCL-04: Kafka consumer lag > 60 s on eel.portal.gazette.v1 topic; page alert fired; backpressure applied; no OOM; slow-consumer chaos test drains to zero lag.

Connectivity and operations:
- EC-CON-08: DNS poisoned or unreachable when submitting to Government Printer; hardcoded fallback Gov Printer endpoint used; circuit breaker activates; DNSChaos test passes.
- EC-CON-11: Kafka ISR shrinks below quorum mid-gazette-publication; outbox accumulates; no gazette event lost; ISR recovers and outbox drains; kill-2-brokers test passes.
- EC-OPS-10: PITR restore during petition window; RPO <= 15 min; RTO <= 30 min; restored gazette audit log verified against transparency log entry; DR harness passes.

Human factors:
- EC-HUM-08: 11 languages including long Swahili and Arabic strings; no truncation in notice templates at 150% pseudo-locale expansion; ICU plurals correct; font fallback renders all scripts.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Rule engine date computation, FSM, notice versioning | nextest | M52-UNIT- |
| property | Rule engine boundary dates; notice amendment immutability | proptest | M52-PROP- |
| contract | Gov Printer double; M51 result event | Pact | M52-PACT- |
| integration | Full gazette workflow; auto-notice from events | nextest + testcontainers | M52-INT- |
| e2e | Public calendar; admin publish; amendment | Playwright | M52-E2E- |
| load | Calendar portal at 5M/hr | k6 + Gatling | M52-LOAD- |
| security | ZAP; cache-poisoning test; log scanner | ZAP + Semgrep | M52-SEC- |
| i18n | 11 locales; iCal localised dates | custom | M52-I18N- |
| a11y | axe on public portal | axe-core | M52-A11Y- |

Tags: `#[trace("FR-M52-002","AC-M52-01","EC-LEG-02")]`

## Runbooks and dashboards

- docs/runbooks/portal-svc-degradation.md (extend with gazette section)
- docs/runbooks/gov-printer-failure.md (new)
- platform/observability/portal-svc/gazette-dashboard.json
- platform/observability/portal-svc/slo-alerts.yaml

## Open questions

- Government Printer API specification and SLA not yet received; double must gate development.
- Whether amendment to a published gazette notice requires a new notice number or an addendum; clarification needed from Government Printer.
