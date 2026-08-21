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

1. FR-M58-001: Create versioned draft statutory report (parliamentary or judiciary type).
2. FR-M58-002: Version control with full diff history for draft reports.
3. FR-M58-003: Dual-sign approval for statutory report submission.
4. FR-M58-004: Submit statutory report to Parliament (select committee / speaker).
5. FR-M58-005: Record acknowledgement from Parliament with reference number.
6. FR-M58-006: Submit testimony for parliamentary hearing with hearing date and committee reference.
7. FR-M58-007: Create Supreme Court pre-registration package on result declaration.
8. FR-M58-008: Generate judiciary handover bundle with declaration, certificates, and evidence.
9. FR-M58-009: Record SC acknowledgement with case reference.
10. FR-M58-010: Admin list of reports with filter by type, election, status.
11. FR-M58-011: Audit trail of all report transitions.
12. FR-M58-012: Export reports as signed PDF.
13. FR-M58-013: Alert when report submission deadline approaching.

## Acceptance criteria

1. AC-M58-01: Draft versioning preserves all historical versions; diff available; integration test.
2. AC-M58-02: Statutory report dual-sign required; SpiceDB policy test.
3. AC-M58-03: Judiciary handover triggered automatically on result_declared.v1; e2e test.
4. AC-M58-04: API p99 within routine 99.5% SLA.

## Applicable edge cases

**Legal:**
- EC-LEG-08: evidence request at T+6d 23h; 24 h clock; not freeze-blocked; timed test.
- EC-LEG-04: recount ordered; report version updated via compensating event.

**Data:**
- EC-DAT-01: duplicate report submission; idempotent.

**Security:**
- EC-SEC-05: injection on committee ref or testimony content.
- EC-SEC-16: no classified content in logs.

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
