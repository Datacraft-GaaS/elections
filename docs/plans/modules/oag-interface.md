# M59 oag-interface

| field | value |
|---|---|
| M-number | M59 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.1 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M59-oag-interface.md |

## Purpose

Provides the Office of the Auditor-General with scoped, time-boxed audit engagement access to financial, inventory, payroll, and archive data packages, with evidence scope management and response tracking.

## Surfaces and languages

| surface | technology | path under services/governance-svc/ |
|---|---|---|
| OAG auditor portal | Django | django/governance-admin/ (OAG role) |
| Internal API | Rust (axum) | crates/oag/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| AuditEngagement | id, engagement_ref, financial_year, scope_description, lead_auditor, status (open/evidence-gathering/responded/closed), opened_at, closed_at | C | T1-consensus |
| EvidenceScope | id, engagement_id, module_ref, data_type, time_range_from, time_range_to, approved_by, approved_at | C | T1-consensus |
| EvidencePackage | id, scope_id, package_url, hash, generated_at, expires_at | C | T1-consensus |
| AuditResponse | id, engagement_id, finding_ref, response_text, responded_by, responded_at | C | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/oag/engagements | 3 accredited | JWT+SpiceDB | n/a | list audit engagements |
| POST /v1/oag/engagements | 3 accredited | JWT+SpiceDB | yes | create engagement |
| POST /v1/oag/engagements/{id}/scopes | 3 accredited | JWT+SpiceDB | yes | add evidence scope with approval |
| POST /v1/oag/engagements/{id}/scopes/{scope_id}/package | 3 accredited | JWT+SpiceDB | yes | generate evidence package |
| GET /v1/oag/engagements/{id}/scopes/{scope_id}/package | 3 accredited | JWT+SpiceDB | n/a | download evidence package (time-boxed URL) |
| POST /v1/oag/engagements/{id}/responses | 3 accredited | JWT+SpiceDB | yes | submit management response |
| PUT /v1/oag/engagements/{id}/close | 3 accredited | JWT+SpiceDB | yes | close engagement |

OpenAPI: packages/openapi-contracts/governance/oag-interface.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.audit_engagement_opened.v1 | emits | eel.governance.audit.v1 | operational | no | at-least-once idempotent |
| eel.governance.evidence_package_generated.v1 | emits | eel.governance.audit.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M54 budget-expenditure | internal | crates/oag/ | yes | advisory; financial data extract |
| M14 reproducibility-bundle | internal | crates/oag/ | yes | advisory; bundle package |
| M20 inventory-management | internal | crates/oag/ | yes | advisory; inventory extract |
| M55 hr-payroll | internal | crates/oag/ | yes | advisory; payroll extract |
| M69 long-term-archive | internal | crates/oag/ | yes | advisory; archive package |

## ML and AI

none

## Dependencies

- Modules: M54, M14, M20, M55, M69
- Platform: P01 (SpiceDB scoped access), P04 (Kafka outbox), MinIO (evidence packages)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py

## Feature checklist

1. FR-M59-001: Create audit engagement with OAG reference and financial year.
2. FR-M59-002: Define evidence scope per module with time range and approval.
3. FR-M59-003: Generate evidence package as signed MinIO bundle.
4. FR-M59-004: Provide time-boxed signed download URL for evidence package (expires ≤48 h).
5. FR-M59-005: Revoke package URL before expiry if scope changes.
6. FR-M59-006: Scope changes require dual approval (IEBC Finance + OAG lead).
7. FR-M59-007: Guard against scope creep: only approved data types accessible per scope.
8. FR-M59-008: Submit management response to OAG finding.
9. FR-M59-009: Close engagement with final response status.
10. FR-M59-010: OAG auditor portal shows scoped data only; no cross-engagement leak.
11. FR-M59-011: Audit trail of all package generation and download events.
12. FR-M59-012: Alert on package approaching expiry without download.
13. FR-M59-013: All evidence packages hashed; hash stored in transparency log.

## Acceptance criteria

1. AC-M59-01: Evidence package URL expires after ≤48 h; access denied after expiry; test.
2. AC-M59-02: Scope creep blocked: request for data type not in scope returns 403; test.
3. AC-M59-03: Dual approval required for scope change; SpiceDB policy test.
4. AC-M59-04: Package hash matches content; tamper detected; test.
5. AC-M59-05: OAG auditor cannot access other engagement data; isolation test.
6. AC-M59-06: API p99 within routine 99.5% SLA.

## Applicable edge cases

**Security:**
- EC-SEC-07: path traversal on package URL; sanitised.
- EC-SEC-16: evidence package contents not in logs.
- EC-IDN-09: dual-control scope approver revoked mid-flow; re-validate at commit.

**Data:**
- EC-DAT-08: large evidence package (financial + inventory export); streaming; bounded memory.

**Connectivity:**
- EC-CON-05: flapping MinIO; package generation retried; no partial packages.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Scope guard, URL expiry, package hash | nextest | M59-UNIT- |
| property | Scope isolation invariant; URL expiry boundary | proptest | M59-PROP- |
| contract | M54 financial extract; M69 archive package | Pact | M59-PACT- |
| integration | Full engagement lifecycle; scope and package | nextest + testcontainers | M59-INT- |
| e2e | OAG portal; scope creation; package download | Playwright | M59-E2E- |
| security | ZAP; path traversal; log scanner | ZAP + Semgrep | M59-SEC- |

Tags: `#[trace("FR-M59-004","AC-M59-01","EC-SEC-07")]`

## Runbooks and dashboards

- docs/runbooks/governance-svc-degradation.md (extend with OAG section)
- platform/observability/governance-svc/oag-dashboard.json
- platform/observability/governance-svc/slo-alerts.yaml

## Open questions

- Scope creep risk: OAG may request data beyond initially agreed scope; escalation process and technical boundary enforcement to be agreed with IEBC legal.
- Whether evidence packages must be retained for a minimum period post-engagement; retention policy to align with KNADS requirements.
