# M62 transmission-legal-basis-audit

| field | value |
|---|---|
| M-number | M62 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.1 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M62-transmission-legal-basis-audit.md |

## Purpose

Verifies 100% of RTS transmission events against the statutory legal rules engine, producing per-event compliance verdicts and party-level summaries, with full audit SDK coverage as the primary acceptance criterion.

## Surfaces and languages

| surface | technology | path under services/governance-svc/ |
|---|---|---|
| Compliance admin console | Django | django/governance-admin/ |
| Party compliance portal | Django (party role) | django/governance-admin/ (party subdomain) |
| Internal API | Rust (axum) | crates/legal-audit/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| TransmissionEvent | id, form34a_event_id (FK to M10 event), station_id, transmitted_at, legal_rule_set_version, verdict (compliant/non-compliant/disputed), verdict_reason | C | T1-audit |
| ComplianceVerdict | id, transmission_event_id, rule_code, passed, detail | C | T1-audit |
| LegalRule | id, rule_code, description, statutory_basis, version, effective_from | C | T1-consensus |
| PartySummary | id, election_id, party_id, total_transmissions, compliant_count, non_compliant_count, computed_at | D | T2 |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/legal-audit/rules | 1 signed-static | public | n/a | current rule set |
| GET /v1/legal-audit/verdicts | 3 accredited | JWT+SpiceDB | n/a | list verdicts with filter |
| GET /v1/legal-audit/verdicts/{id} | 3 accredited | JWT+SpiceDB | n/a | verdict detail |
| GET /v1/legal-audit/party-summary | 3 accredited | JWT+SpiceDB | n/a | party compliance summary |
| POST /v1/legal-audit/verdicts/{id}/dispute | 3 accredited | JWT+SpiceDB | yes | party disputes verdict |
| GET /v1/legal-audit/verdicts/export | 4 bulk | JWT+SpiceDB | n/a | full audit export |

OpenAPI: packages/openapi-contracts/governance/transmission-legal-basis-audit.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.transmission_verdict_issued.v1 | emits | eel.governance.legal_audit.v1 | operational | no | at-least-once idempotent |
| eel.rts.form34a_captured.v1 | consumes | eel.rts.form34a.v1 | event-source | yes | at-least-once idempotent |
| eel.rts.form34a_amended.v1 | consumes | eel.rts.form34a.v1 | event-source | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M10 rts-backend (transmission events) | internal | crates/legal-audit/ | yes | hard dependency; audit runs on event stream |
| M40 election-court-it-interface | internal | crates/legal-audit/ | yes | advisory; non-compliant bundle available |
| P05 transparency log | internal | crates/legal-audit/ | yes (SoftHSM) | degraded; verdict still issued locally |

## ML and AI

none

## Dependencies

- Modules: M10 (transmission events), M40 (court bundle)
- Platform: P04 (Kafka), P05 (transparency log for verdict hashing)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py

## Feature checklist

1. FR-M62-001: Consume every form34a_captured and form34a_amended event from M10.
2. FR-M62-002: Apply legal rule set to each transmission event; produce per-event verdict.
3. FR-M62-003: Verdict covers: timing (within statutory window), originator authentication, form completeness, signature validity.
4. FR-M62-004: Store per-rule-code compliance detail for each verdict.
5. FR-M62-005: Achieve 100% coverage of all transmission events in election cycle; alert if gap.
6. FR-M62-006: Hash each verdict and submit to transparency log.
7. FR-M62-007: Party portal shows party-level compliance summary.
8. FR-M62-008: Party can dispute a non-compliant verdict with reason.
9. FR-M62-009: Dispute triggers manual review workflow; reviewer assigns final verdict.
10. FR-M62-010: Non-compliant summary available as court bundle item in M40.
11. FR-M62-011: Legal rule set versioned; rule drift alerting when rule set changes post-election-start.
12. FR-M62-012: Export full audit as signed XLSX and JSON.
13. FR-M62-013: Admin dashboard with compliance rate by party and station.
14. FR-M62-014: Audit trail of all verdicts and disputes.
15. FR-M62-015: Rule set published publicly; signed; CDN-served.

## Acceptance criteria

1. AC-M62-01: 100% of form34a_captured events receive a verdict within 60 s of event arrival; coverage metric alert if gap.
2. AC-M62-02: Verdict hash included in transparency log; inclusion proof verifiable; test.
3. AC-M62-03: Rule drift detected when LegalRule changes after election activation; alert within 5 min.
4. AC-M62-04: Dispute workflow routes to manual review; idempotent; double-fire test.
5. AC-M62-05: API p99 within routine 99.5% SLA; consumer lag ≤60 s under load.

## Applicable edge cases

**Time:**
- EC-TIM-06: event time < causation time (late transmission); verdict issued with skew annotation.
- EC-TIM-08: transmission window boundary at statutory close; +-1 s boundary test.

**Data:**
- EC-DAT-01: duplicate form34a event; idempotent verdict; no double-count.
- EC-DAT-03: out-of-order events; buffer until gap filled or DLQ.

**Security:**
- EC-SEC-01: signature mismatch on consumed event; verdict marked non-compliant with tamper reason.
- EC-SEC-13: split-view on transparency log; witness divergence alert.

**Scale:**
- EC-SCL-04: consumer lag >60 s; backpressure; alert.
- EC-SCL-08: Valkey eviction loses idempotency; PG-durable verdict store.

**Legal:**
- EC-LEG-09: T-60 auto-referral race; single referral; idempotent scheduler.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Rule engine per-code, verdict FSM, dispute routing | nextest | M62-UNIT- |
| property | 100% coverage invariant; idempotency; timing boundary | proptest | M62-PROP- |
| contract | M10 form34a event; M40 court bundle | Pact | M62-PACT- |
| integration | Full verdict pipeline; transparency log inclusion | nextest + testcontainers | M62-INT- |
| e2e | Party portal compliance summary; dispute | Playwright | M62-E2E- |
| load | Consumer at 60k event/s RTS peak | k6 | M62-LOAD- |
| security | ZAP; log scanner; signature mismatch verdict | ZAP + Semgrep | M62-SEC- |

Tags: `#[trace("FR-M62-005","AC-M62-01","EC-TIM-08")]`

## Runbooks and dashboards

- docs/runbooks/governance-svc-degradation.md (extend with legal-audit section)
- docs/runbooks/transmission-coverage-gap.md (new; trigger when 100% coverage dips)
- platform/observability/governance-svc/legal-audit-dashboard.json
- platform/observability/governance-svc/slo-alerts.yaml (coverage gap alert, lag alert)

## Open questions

- Rule drift: the upstream sw/ text may reference rule sets that conflict with ADR constraints; a rule engine config review is needed before election cycle activation.
- Whether disputed verdicts that remain unresolved by declaration day require automatic escalation to court or remain in administrative limbo.
