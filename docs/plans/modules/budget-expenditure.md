# M54 budget-expenditure

| field | value |
|---|---|
| M-number | M54 |
| Home service | support-admin-svc (ADR-0020) |
| Owning work package | WP-5.6 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M54-budget-expenditure.md |

## Purpose

Manages the Commission election budgets, procurement records, payment vouchers, and CoB/OAG financial reports with IFMIS and PPRA integration doubles, enforcing Separation of Duties (SoD) for payment authorisation.

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

Budget management:

FR-M54-001: Create election budget document linked to election ID and financial year; initial status DRAFT (spec FR-M54-001).
FR-M54-002: Budget document requires dual Commissioner sign-off to transition from DRAFT to APPROVED; single signature attempt blocked (spec FR-M54-001).
FR-M54-003: Add budget lines by programme code with description and allocated KES amount (spec FR-M54-001).
FR-M54-004: Lock approved budget version; subsequent reallocations require amendment workflow (spec FR-M54-011).
FR-M54-005: Amendment workflow: reallocation between lines requires CEO approval above threshold; mandatory reason; amendment record with approval reference (spec FR-M54-011).
FR-M54-006: Track budget line committed amount as running sum of active procurement contract values (spec FR-M54-002).
FR-M54-007: Track budget line spent amount as running sum of paid payment vouchers (spec FR-M54-002).
FR-M54-008: Compute and display variance (allocated minus spent) per line; updated in real time on every voucher payment (spec FR-M54-002).
FR-M54-009: Amber alert when committed plus spent exceeds 85% of allocated on any line; alert emitted to eel-iems-54-alerts.
FR-M54-010: Red alert when committed plus spent reaches or exceeds 100% of allocated; further commitments against that line blocked (spec FR-M54-002).
FR-M54-011: Budget utilisation chart on admin dashboard refreshed every 60 seconds; donut chart per cost head.
FR-M54-012: Budget document status lifecycle: DRAFT, APPROVED, ACTIVE, AMENDED, CLOSED; each transition logged with actor and timestamp.

Procurement:

FR-M54-013: Create procurement record with description, estimated KES, PPRA reference, and procurement method (spec FR-M54-003).
FR-M54-014: Validate procurement method against PPADA 2015 threshold rules from config table: Open Tender above configured KES ceiling; Direct Procurement requires written justification field (spec FR-M54-003).
FR-M54-015: Link procurement record to budget line; line committed amount updated on record creation.
FR-M54-016: Route procurement approval per delegated authority matrix: amounts above CEO threshold require Commissioner countersign (spec FR-M54-003).
FR-M54-017: Submit annual and quarterly procurement plan to PPRA portal API; record PPRA acknowledgement reference (spec FR-M54-004).
FR-M54-018: Submit contract award notice to PPRA within statutory window; alert if submission is overdue (spec FR-M54-004).
FR-M54-019: Contract register: supplier name, contract amount KES, start and end dates, PPRA notice reference, payment schedule (spec FR-M54-005).
FR-M54-020: Milestone tracker per contract: planned versus actual delivery dates; alert on missed milestone.
FR-M54-021: Contract insurance and bond expiry alerts: alert 30 days before expiry date.
FR-M54-022: Contract status lifecycle: DRAFT, APPROVED, ADVERTISED, EVALUATED, AWARDED, SIGNED, ACTIVE, COMPLETE, CANCELLED.
FR-M54-023: Procurement record status updates emit eel.finance.budget_approved.v1 or appropriate topic event.
FR-M54-024: PPRA portal adapter circuit breaker: advisory failure mode; procurement record saved locally; alert on 3 consecutive PPRA failures.

Payment vouchers:

FR-M54-025: Create payment voucher against procurement record with invoice reference and amount KES (spec FR-M54-006).
FR-M54-026: Voucher amount validated against remaining available budget line balance; exceeding balance returns 422 with remaining balance in body (spec FR-M54-017).
FR-M54-027: SoD: voucher raiser cannot be the first approver; SpiceDB policy enforced at API level (spec NFR-M54-009).
FR-M54-028: SoD: first approver cannot be counter-approver; second approval from distinct officer required (spec NFR-M54-009).
FR-M54-029: Voucher amount tier determines approval chain: Finance Officer below KES threshold; CEO above; Commissioner above top threshold (spec FR-M54-006).
FR-M54-030: Submit approved voucher to IFMIS API for payment execution; record IFMIS payment reference on confirmation (spec FR-M54-006).
FR-M54-031: IFMIS circuit breaker: retry up to 3 times with exponential backoff; voucher stays PENDING_IFMIS on exhaustion; alert emitted (spec FR-M54-006).
FR-M54-032: Record IFMIS payment confirmation reference and payment date on voucher; status transitions to PAID.
FR-M54-033: Voucher rejection: rejector must provide mandatory reason; voucher returned to DRAFT for correction.
FR-M54-034: Payment voucher idempotent on invoice reference; duplicate submission returns existing voucher record.
FR-M54-035: No soft-delete on any voucher or budget record; all state changes append-only in audit log (spec NFR-M54-008).

Controller of Budget reporting:

FR-M54-036: Generate quarterly CoB progress report per Controller of Budget Act format: budget versus actual per vote head, procurement status, pending payments, narrative field (spec FR-M54-008).
FR-M54-037: CoB report auto-generated 5 business days before quarterly submission deadline; Finance Officer notified by alert (spec FR-M54-008).
FR-M54-038: CoB report exported as PDF with HSM signature and as XLSX for spreadsheet review (spec FR-M54-008).
FR-M54-039: CoB submission recorded with timestamp, submitter ID, and CoB portal acknowledgement reference.
FR-M54-040: CoB portal API adapter: primary submission via API; fallback is signed PDF manual upload with upload timestamp recorded.

OAG audit package:

FR-M54-041: Compile post-election audit-readiness package: vote-book summary, contract register, payment vouchers, procurement plans, PPRA notices (spec FR-M54-009).
FR-M54-042: Audit package compilation uses automated checklist; red blocking gate if any required document is missing.
FR-M54-043: Audit package HSM-signed by CEO and Commissioner before transmission to OAG secure portal (spec FR-M54-009).
FR-M54-044: OAG delivery confirmation timestamp and reference stored on audit package record.
FR-M54-045: 30-day pre-deadline compilation run auto-populates checklist and flags missing items.
FR-M54-046: Read-only OAG view via M59 interface for live audit engagements; OAG cannot modify any record.

Asset register:

FR-M54-047: Capital asset registration: serial number, description, value KES, custodian officer ID, linked procurement record.
FR-M54-048: Asset register feeds M20 Inventory Management with kit and equipment serial numbers.
FR-M54-049: Asset disposal event records disposal date, method, and Commissioner approval reference.

By-election and subsidiary cost centres:

FR-M54-050: Per-by-election subsidiary cost centre linked to M50 event; own budget versus actual view; scoped to event constituency set (spec FR-M54-007).
FR-M54-051: Post-by-election cost summary auto-generated as M50 output artefact.
FR-M54-052: M55 payroll run totals automatically feed into personnel budget line actual amount via eel.hr.payroll.v1 event.

Dashboard and reporting:

FR-M54-053: Admin dashboard: donut chart of budget versus actual by cost head; amber and red line indicators; total committed versus available.
FR-M54-054: Procurement pipeline Kanban view: stages from requisition to contract; PPADA method badge; PPRA submission status column (spec FR-M54-003).
FR-M54-055: WCAG 2.2 AA on all Finance Officer and Procurement Officer console screens (spec NFR-M54-005).
FR-M54-056: EN primary language for all financial and legal documents; SW for internal dashboard labels (spec NFR-M54-006).

Security and compliance:

FR-M54-057: All financial record writes use parameterised queries; SQL injection surface eliminated; Semgrep rule enforced in CI.
FR-M54-058: SpiceDB authorisation enforced on all write and approval endpoints; role violations return 403 problem+json.
FR-M54-059: Segregation of duties violations logged and alerted immediately; zero-tolerance policy enforced at API level; no override without Commissioner countersign.

Monitoring:

FR-M54-060: Module Grafana dashboard eel-iems-54: budget utilisation per line, voucher queue depth, IFMIS retry count, CoB report status, OAG package checklist completeness.
FR-M54-061: Alert on IFMIS submission failure after 3 retries; alert on PPRA submission overdue; both route to eel-iems-54-alerts Alertmanager group.

## Acceptance criteria

AC-M54-01: SoD enforced: same user cannot raise and approve a payment voucher; attempt returns 403; SpiceDB policy test.
AC-M54-02: Same user cannot first-approve and counter-approve a voucher; attempt returns 403; unit test.
AC-M54-03: Voucher blocked when budget line balance is exhausted; returns 422 with remaining balance in body; property test on 10k synthetic rows.
AC-M54-04: IFMIS circuit breaker opens after 3 consecutive failures; voucher stays PENDING_IFMIS; no duplicate payment on retry; integration test with IFMIS double.
AC-M54-05: CoB report export row sums match vote-book totals to KES zero variance; property test on 10k synthetic voucher rows.
AC-M54-06: Audit trail contains an immutable entry for every budget, procurement, and voucher state transition; no gap; integration test.
AC-M54-07: API p99 <= 500 ms at routine 99.5% availability class under 3x peak Finance Officer concurrency; k6 load test.
AC-M54-08: Budget versus actual reconciliation completes within 60 s on full election dataset; performance test (spec NFR-M54-001).
AC-M54-09: No soft-delete possible on any financial record; DELETE attempt returns 405; unit test.
AC-M54-10: 100% of test procurement records submitted to PPRA within statutory window in integration test; PPRA double confirms receipt.
AC-M54-11: Dual Commissioner sign-off required to approve budget; single signature attempt returns 403; unit test.
AC-M54-12: OAG audit package HSM-signed; signature verifiable; transmission confirmed within T+60 days post-election; integration test with OAG secure portal double.
AC-M54-13: Financial amounts and IFMIS references absent from log output at WARNING level and below; CI log-scanner fixture passes.
AC-M54-14: WCAG 2.2 AA on Finance Officer console; axe-core zero critical violations; keyboard-only task completion verified.
AC-M54-15: CoB report auto-generated 5 business days before quarterly deadline; scheduler integration test confirms trigger timing within 1-minute tolerance.
AC-M54-16: Budget amendment above CEO threshold blocked when initiated by Finance Officer alone; attempt returns 403; unit test.
AC-M54-17: Injection attempt on vendor ID or procurement description fields blocked; ZAP scan and Semgrep both pass in CI.

## Applicable edge cases

EC-DAT-01: Duplicate payment voucher submitted with same invoice reference; idempotent; existing voucher returned with 200.
EC-DAT-02: Same voucher idempotency key submitted with different invoice amount; 422 idempotency-key-reuse returned; no second voucher created.
EC-DAT-05: Poison message on eel.finance.voucher.v1 topic (voucher amount exceeds entire budget total); DLQ with reason; Kafka partition continues processing.
EC-DAT-07: Partial batch failure during CoB report generation; per-line status report generated; valid lines committed to report.
EC-DAT-10: Zero-balance budget line; division-by-zero guard in utilisation percentage chart; renders "0%" without error.
EC-DAT-12: Debezium outbox slot lag on finance.voucher topic; alert at 60 s lag; outbox retained; disk guard enforced; no silent data loss.
EC-DAT-14: Oversized audit package event exceeds 1 MB; binary blob pushed to MinIO; SHA-256 hash stored in event; unit test with fuzzed sizes.
EC-SEC-05: SQL injection attempted on vendor ID, procurement description, or invoice reference; parameterised queries block; Semgrep and ZAP both pass in CI.
EC-SEC-06: SSRF via IFMIS or PPRA URL configuration field; allowlist enforced; private IP ranges refused; rebinding resolver test.
EC-SEC-09: Rate-limit evasion on voucher approval endpoint; key includes subject, device ID, and /64 subnet; priority shedding enforced.
EC-SEC-16: Financial amounts, IFMIS references, and contractor personal data absent from all log lines; log-scanner CI fixture passes.
EC-IDN-02: JWT expires mid-voucher-approval request; in-flight request completes; next request returns 401 with silent-refresh hint.
EC-IDN-09: Dual-control counter-approver account revoked after first approval; second approval re-validates both identities at commit; operation rejected if revoked.
EC-IDN-10: SpiceDB unavailable during voucher approval; fail closed; 503 problem+json returned; no partial approval committed.
EC-CON-04: Site partition between primary and mirror during voucher submission; Patroni fencing; no split-brain; RPO <= 5 min confirmed by Chaos Mesh test.
EC-CON-05: Flapping IFMIS API; exponential backoff with jitter; retry budget enforced; no duplicate payment submission to IFMIS.
EC-CON-11: Kafka ISR shrink during voucher event publish; producer blocks; outbox accumulates; alert paged; no event loss.
EC-SCL-04: Consumer lag on eel.finance.voucher.v1 topic exceeds 60 s; alert paged; backpressure applied; no OOM on consumer pod.
EC-SCL-08: Valkey eviction during voucher idempotency cache lookup; PG-durable idempotency store used as source of truth; Valkey is cache only; flush-mid-test confirms no duplicate.
EC-TIM-06: Voucher event timestamp precedes causation timestamp due to clock skew; accepted; audit skew anomaly logged; projection remains correct and independent of wall-clock order.
EC-LEG-09: CoB auto-trigger scheduler fires twice due to race condition; single submission only; idempotent scheduler; double-fire integration test.
EC-LEG-10: Duplicate IFMIS payment callback; single payment credited; idempotent refund path; no double-count in budget actuals.
EC-OPS-08: Disk full during audit package compilation; 507 returned; alert emitted; no corruption; bounded queue prevents unbounded growth.
EC-OPS-10: PITR restore during audit engagement; RPO and RTO drill; restore logged to P05 audit chain; no data gap in financial records.
EC-SEC-04: KEK rotation mid-election cycle on encrypted financial records; online re-wrap; old key readable until rotation complete; full audit trail of re-wrap operation.
EC-HUM-08: Long Swahili dashboard labels in budget line descriptions; no truncation; ICU plurals applied; font fallback stack tested.
EC-HUM-05: Finance Officer temporarily unavailable; delegated authority workflow activated; assistant recorded with consent; no approval bypass.

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
