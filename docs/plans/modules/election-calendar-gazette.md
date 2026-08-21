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

1. FR-M52-001: Create and publish election calendar with all statutory milestones.
2. FR-M52-002: Statutory rule engine computes milestone dates from election date + rule offsets.
3. FR-M52-003: Enforce mandatory milestones; alert if not published by deadline.
4. FR-M52-004: Draft, approve, and publish gazette notice workflow with dual approval.
5. FR-M52-005: Sign published notice with HSM-anchored key.
6. FR-M52-006: Submit notice to Government Printer API; track submission reference.
7. FR-M52-007: Public calendar in JSON and iCal format with statutory basis per event.
8. FR-M52-008: Notice content available as HTML and PDF.
9. FR-M52-009: Amend published notice via compensating event with reason; original immutable.
10. FR-M52-010: Auto-create gazette notice on result_declared.v1 from M51.
11. FR-M52-011: Auto-create gazette notice on referendum_activated.v1 from M49.
12. FR-M52-012: Auto-create gazette notice on by_election_created.v1 from M50.
13. FR-M52-013: Email subscription for calendar events (hashed subscriber email, no PII in logs).
14. FR-M52-014: RSS feed for gazette notices.
15. FR-M52-015: Admin list of notices with filter by election, status, notice number.
16. FR-M52-016: Audit trail of notice lifecycle transitions.
17. FR-M52-017: i18n of calendar event labels and notice templates in 11 languages.
18. FR-M52-018: WCAG 2.2 AA on public portal.
19. FR-M52-019: Notice PDF generated with statutory formatting; version-stamped.
20. FR-M52-020: Calendar accessible via CDN edge; p95 200 ms / p99 500 ms under load.

## Acceptance criteria

1. AC-M52-01: Statutory rule engine computes correct milestone dates for boundary cases (election_date minus rule offset); property test with 5 rule types.
2. AC-M52-02: Gazette notice signed; Gov Printer double returns success; notice status set to published.
3. AC-M52-03: Gov Printer unavailable triggers circuit breaker; notice queued; retry on reconnect.
4. AC-M52-04: Auto-notice on result_declared.v1; emitted idempotently; e2e test.
5. AC-M52-05: Amendment creates compensating event; original notice immutable; test.
6. AC-M52-06: Public calendar p95 200 ms at routine 99.5% SLA.
7. AC-M52-07: 11 locales snapshot pass; iCal feed localised dates correct.

## Applicable edge cases

**Legal:**
- EC-LEG-02: court-ordered postponement; signed calendar amendment; portal notice.
- EC-TIM-08: petition window T+28d midnight; calendar event boundary test.

**Security:**
- EC-SEC-12: cache poisoning of gazette snapshots; key includes signature version; client verifies.
- EC-SEC-16: subscriber emails hashed; not in logs.

**Scale:**
- EC-SCL-01: 5M visitors/hr on calendar portal; 95% edge; origin p99 500 ms.
- EC-SCL-02: thundering herd on STH; SDK jitter; ETag 304.

**Connectivity:**
- EC-CON-08: DNS poisoned; hardcoded fallback Gov Printer endpoint.

**Human:**
- EC-HUM-08: 11 languages; no truncation in notice templates.

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
