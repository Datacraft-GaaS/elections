# M54 budget-expenditure

| field | value |
|---|---|
| M-number | M54 |
| Home service | support-admin-svc (ADR-0020) |
| Owning work package | WP-5.6 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M54-budget-expenditure.md |

## Purpose

Manages IEBC election budgets, procurement records, payment vouchers, and CoB/OAG financial reports with IFMIS and PPRA integration doubles, enforcing Separation of Duties (SoD) for payment authorisation.

## Surfaces and languages

| surface | technology | path under services/support-admin-svc/ |
|---|---|---|
| Finance admin console | Django | django/finance/ |
| Internal API | Rust (axum) | crates/budget/ |
| IFMIS adapter | Python | py/ifmis/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| Budget | id, election_id, financial_year, total_kes, approved_by, status (draft/approved/active/closed) | C | T1-consensus |
| BudgetLine | id, budget_id, programme_code, description, allocated_kes, committed_kes, spent_kes | C | T1-consensus |
| ProcurementRecord | id, budget_line_id, vendor_id, ppra_ref, amount_kes, procurement_method, status | C | T1-audit |
| PaymentVoucher | id, procurement_id, amount_kes, approved_by, counter_approved_by, ifmis_ref, status (pending/approved/paid/rejected) | B | T1-audit |
| CoBReport | id, budget_id, reporting_period, total_committed, total_spent, variance, submitted_at | D | T2 |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/budgets | 3 accredited | JWT+SpiceDB | n/a | list budgets |
| POST /v1/budgets | 3 accredited | JWT+SpiceDB | yes | create budget |
| PUT /v1/budgets/{id}/approve | 3 accredited | JWT+SpiceDB | yes | approve budget (dual sign) |
| GET /v1/budgets/{id}/lines | 3 accredited | JWT+SpiceDB | n/a | budget lines |
| POST /v1/budgets/{id}/lines | 3 accredited | JWT+SpiceDB | yes | add budget line |
| POST /v1/procurement | 3 accredited | JWT+SpiceDB | yes | create procurement record |
| POST /v1/vouchers | 3 accredited | JWT+SpiceDB | yes | create payment voucher |
| PUT /v1/vouchers/{id}/approve | 3 accredited | JWT+SpiceDB | yes | first approval (SoD) |
| PUT /v1/vouchers/{id}/counter-approve | 3 accredited | JWT+SpiceDB | yes | second approval (SoD) |
| PUT /v1/vouchers/{id}/pay | 3 accredited | JWT+SpiceDB | yes | submit to IFMIS for payment |
| GET /v1/reports/cob | 3 accredited | JWT+SpiceDB | n/a | CoB report |
| GET /v1/reports/cob/export | 4 bulk | JWT+SpiceDB | n/a | OAG-ready export |

OpenAPI: packages/openapi-contracts/support-admin/budget-expenditure.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.finance.voucher_approved.v1 | emits | eel.finance.voucher.v1 | operational | no | at-least-once idempotent |
| eel.finance.voucher_paid.v1 | emits | eel.finance.voucher.v1 | operational | no | at-least-once idempotent |
| eel.finance.budget_approved.v1 | emits | eel.finance.budget.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| IFMIS | ifmis | integrations/ifmis/ | yes | circuit breaker; voucher queued for retry |
| PPRA procurement registry | ppra | integrations/ (tier B) | yes | advisory; procurement proceeds |
| M47 nomination fees (cost reference) | internal | crates/budget/ | yes | advisory |
| M55 hr-payroll (payroll budget lines) | internal | crates/budget/ | yes | advisory |
| M59 oag-interface (audit extract) | internal | crates/budget/ | yes | advisory; export on request |

## ML and AI

none

## Dependencies

- Modules: M47 (fee revenue), M55 (payroll cost), M59 (OAG audit)
- Platform: P01 (SpiceDB SoD policy), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py

## Feature checklist

1. FR-M54-001: Create and approve budget with Commissioner dual-sign.
2. FR-M54-002: Manage budget lines by programme code.
3. FR-M54-003: Track committed and spent amounts per line in real time.
4. FR-M54-004: Alert when line committed exceeds allocated.
5. FR-M54-005: Create procurement record with PPRA reference.
6. FR-M54-006: Validate procurement method against PPRA rules.
7. FR-M54-007: Create payment voucher against procurement record.
8. FR-M54-008: Enforce SoD: approver and counter-approver must be different persons.
9. FR-M54-009: Submit approved voucher to IFMIS for payment.
10. FR-M54-010: Track IFMIS payment reference on voucher.
11. FR-M54-011: Generate CoB report by period with committed/spent/variance.
12. FR-M54-012: Export OAG-ready CoB report as XLSX and PDF.
13. FR-M54-013: Admin dashboard with budget utilisation chart per programme.
14. FR-M54-014: Admin list of vouchers with filter by status, period, amount.
15. FR-M54-015: Audit trail of all voucher and budget transitions.
16. FR-M54-016: Alert on IFMIS submission failure after three retries.
17. FR-M54-017: Reject voucher amount exceeding available budget line balance.
18. FR-M54-018: Virement (budget line transfer) with dual approval and audit reason.
19. FR-M54-019: Read-only OAG view via M59 for audit engagements.

## Acceptance criteria

1. AC-M54-01: SoD enforced at API level; same user cannot both approve and counter-approve; SpiceDB policy test.
2. AC-M54-02: Voucher blocked when budget line balance exhausted; property test.
3. AC-M54-03: IFMIS circuit breaker triggers on three consecutive failures; voucher stays pending; test.
4. AC-M54-04: CoB report export matches sum of voucher rows; property test on 10k synthetic rows.
5. AC-M54-05: Audit trail complete for every state transition; integration test.
6. AC-M54-06: API p99 latency within routine 99.5% SLA.

## Applicable edge cases

**Security:**
- EC-IDN-09: dual-control approver revoked mid-flow; re-validate both at commit.
- EC-SEC-05: injection on vendor ID or procurement fields.
- EC-SEC-16: financial amounts and IFMIS refs not exposed in logs at wrong level.

**Data:**
- EC-DAT-01: duplicate voucher submission; idempotent.
- EC-DAT-10: zero-balance budget line; no division errors in utilisation chart.

**Connectivity:**
- EC-CON-05: flapping IFMIS; backoff; retry; no duplicate payment.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Budget FSM, SoD policy, line balance guard | nextest | M54-UNIT- |
| property | CoB sum invariant; SoD enforcement | proptest | M54-PROP- |
| contract | IFMIS double; PPRA double | Pact | M54-PACT- |
| integration | Full voucher workflow; IFMIS retry | nextest + testcontainers | M54-INT- |
| e2e | Admin budget creation; voucher approval; CoB export | Playwright | M54-E2E- |
| security | ZAP; SoD bypass attempt; log scanner | ZAP + Semgrep | M54-SEC- |

Tags: `#[trace("FR-M54-008","AC-M54-01","EC-IDN-09")]`

## Runbooks and dashboards

- docs/runbooks/support-admin-svc-degradation.md (extend with budget section)
- docs/runbooks/ifmis-failure.md (new)
- platform/observability/support-admin-svc/budget-dashboard.json
- platform/observability/support-admin-svc/slo-alerts.yaml

## Open questions

- IFMIS API specification and authentication mechanism not yet confirmed; double gates development.
- PPRA procurement threshold values (direct/restricted/open tender) subject to regulatory update; rule engine must be config-driven.
