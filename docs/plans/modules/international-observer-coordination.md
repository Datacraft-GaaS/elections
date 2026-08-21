# M57 international-observer-coordination

| field | value |
|---|---|
| M-number | M57 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.3 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M57-international-observer-coordination.md |

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

1. FR-M57-001: Register observer mission linked to M43 accreditation record.
2. FR-M57-002: Provision isolated workspace per mission (SpiceDB namespace).
3. FR-M57-003: Prevent cross-mission data access; workspace isolation test.
4. FR-M57-004: Schedule pre-election briefings with materials and attendance tracking.
5. FR-M57-005: Submit interim and final observation reports (mission-scoped access).
6. FR-M57-006: Mark report as classified; restricted access to Commission only.
7. FR-M57-007: Publish joint statement with participating mission signatures.
8. FR-M57-008: Public portal lists published joint statements.
9. FR-M57-009: Admin lists all missions with status and accreditation link.
10. FR-M57-010: Mission workspace supports LiveView real-time briefing feed.
11. FR-M57-011: Close mission workspace after election cycle; archive reports.
12. FR-M57-012: Audit trail of all mission events.
13. FR-M57-013: i18n on public joint statement portal.
14. FR-M57-014: WCAG 2.2 AA on public portal.

## Acceptance criteria

1. AC-M57-01: Cross-mission data access blocked; SpaceDB namespace isolation test.
2. AC-M57-02: Mission requires valid M43 accreditation; 422 if not found.
3. AC-M57-03: Joint statement published idempotently; event emitted once.
4. AC-M57-04: API p99 within routine 99.5% SLA.

## Applicable edge cases

**Identity:**
- EC-IDN-10: SpiceDB down; fail closed on workspace access.

**Security:**
- EC-SEC-15: LiveView session hijack; CSRF + session binding.
- EC-SEC-16: classified report content not in logs.

**Data:**
- EC-DAT-01: duplicate report submission; idempotent.

**Human:**
- EC-HUM-08: 11 languages on public portal.

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
