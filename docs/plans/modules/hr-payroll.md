# M55 hr-payroll

| field | value |
|---|---|
| M-number | M55 |
| Home service | support-admin-svc (ADR-0020) |
| Owning work package | WP-5.6 (docs/plans/06-wave-5-governance.md) |
| Size | XL |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M55-hr-payroll.md |

## Purpose

Full lifecycle HR and payroll for IEBC election personnel: job applications, contracts, payroll runs (250k staff in ≤4 h), M-Pesa B2C disbursements, statutory deductions (KRA iTax, NSSF, NHIF/SHIF), and payslip distribution, with Separation of Duties for payroll authorisation.

## Surfaces and languages

| surface | technology | path under services/support-admin-svc/ |
|---|---|---|
| HR admin console | Django | django/hr/ |
| Public job application portal | Astro | astro/hr-public/ |
| Staff payslip portal | Django | django/hr/ |
| Mobile payslip access | Flutter (M33 integration) | apps/personnel (via M33 field) |
| Internal API | Rust (axum) | crates/payroll/ |
| M-Pesa B2C adapter | Python | py/mpesa-b2c/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| JobApplication | id, role_id, applicant_name, national_id_hash, status (applied/shortlisted/selected/rejected), applied_at | B | T2 |
| StaffContract | id, staff_id, role, station_id, start_date, end_date, gross_salary_kes, bank_account_hash | B | T1-consensus |
| PayrollRun | id, election_id, period, status (draft/approved/paid/failed), total_staff, total_gross_kes, started_at, completed_at | C | T1-audit |
| StatutoryDeduction | id, payroll_run_id, staff_id, type (paye/nssf/nhif), amount_kes, reference | B | T1-audit |
| PaymentDisbursement | id, payroll_run_id, staff_id, amount_kes, mpesa_reference, status, disbursed_at | B | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/hr/applications | 2 live-per-user | JWT | yes | submit job application |
| GET /v1/hr/applications/{id} | 2 live-per-user | JWT | n/a | application status |
| POST /v1/hr/contracts | 3 accredited | JWT+SpiceDB | yes | create staff contract |
| PUT /v1/hr/contracts/{id}/terminate | 3 accredited | JWT+SpiceDB | yes | terminate contract |
| POST /v1/payroll/runs | 3 accredited | JWT+SpiceDB | yes | initiate payroll run |
| PUT /v1/payroll/runs/{id}/approve | 3 accredited | JWT+SpiceDB | yes | dual-sign approval |
| PUT /v1/payroll/runs/{id}/disburse | 3 accredited | JWT+SpiceDB | yes | trigger B2C disbursement |
| GET /v1/payroll/runs/{id}/report | 3 accredited | JWT+SpiceDB | n/a | payroll run report |
| GET /v1/payroll/runs/{id}/export | 4 bulk | JWT+SpiceDB | n/a | KRA/NSSF/NHIF statutory returns |
| GET /v1/payroll/payslips/{staff_id}/{period} | 2 live-per-user | JWT | n/a | staff payslip |

OpenAPI: packages/openapi-contracts/support-admin/hr-payroll.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.hr.payroll_run_completed.v1 | emits | eel.hr.payroll.v1 | operational | no | at-least-once idempotent |
| eel.hr.disbursement_confirmed.v1 | emits | eel.hr.payroll.v1 | operational | no | at-least-once idempotent |
| eel.hr.disbursement_failed.v1 | emits | eel.hr.payroll.v1 | operational | no | at-least-once idempotent |
| eel.dispatch.personnel_checked_in.v1 | consumes | eel.dispatch.personnel.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M-Pesa B2C | mpesa-daraja | py/mpesa-b2c/ | yes | circuit breaker; disbursement queued |
| KRA iTax | kra-itax | integrations/kra-itax/ | yes | circuit breaker; return queued |
| NSSF | nssf | integrations/nssf/ | yes | circuit breaker; deduction queued |
| NHIF/SHIF | nhif-shif | integrations/nhif-shif/ | yes | circuit breaker; deduction queued |
| M38 poll-worker LMS (certificate gate) | internal | crates/payroll/ | yes | advisory; contract blocked until certified |
| M33 personnel-dispatch (attendance) | internal | crates/payroll/ | yes | advisory; attendance feeds payroll |
| M54 budget-expenditure (cost) | internal | crates/payroll/ | yes | advisory; payroll cost to budget |

## ML and AI

none

## Dependencies

- Modules: M38 (certification gate), M33 (attendance trigger), M54 (budget cost)
- Platform: P01 (SpiceDB SoD), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py, eel-web-astro

## Feature checklist

1. FR-M55-001: Public job application portal with role listing and application form.
2. FR-M55-002: Application shortlisting and selection workflow in admin.
3. FR-M55-003: Create staff contract with role, station, salary, and bank/M-Pesa details.
4. FR-M55-004: Gate contract on M38 poll-worker training certificate.
5. FR-M55-005: Terminate contract with reason and effective date.
6. FR-M55-006: Initiate payroll run for election period.
7. FR-M55-007: Compute gross pay, PAYE, NSSF, NHIF/SHIF deductions per staff member.
8. FR-M55-008: Enforce SoD: payroll initiator and approver must be different persons.
9. FR-M55-009: Dual-sign payroll run approval.
10. FR-M55-010: Disburse net pay via M-Pesa B2C for 250k staff in ≤4 h.
11. FR-M55-011: Track B2C disbursement status per staff member.
12. FR-M55-012: Retry failed B2C disbursements automatically up to three times.
13. FR-M55-013: Alert operations on disbursement failure after retries.
14. FR-M55-014: Generate KRA PAYE return (P9/P10) for statutory filing.
15. FR-M55-015: Generate NSSF and NHIF/SHIF returns.
16. FR-M55-016: Staff payslip portal with period filter.
17. FR-M55-017: Payslip accessible on M33 mobile app for field staff.
18. FR-M55-018: Export payroll run report as XLSX for M54 budget reconciliation.
19. FR-M55-019: Payroll run status dashboard for HR admin.
20. FR-M55-020: Audit trail of all payroll run state transitions with approver IDs.
21. FR-M55-021: Payroll computation uses integer KES; no floating-point arithmetic.
22. FR-M55-022: Bank account and M-Pesa numbers stored encrypted; hashed in logs.
23. FR-M55-023: i18n on public application portal in 11 languages.
24. FR-M55-024: WCAG 2.2 AA on public and payslip surfaces.
25. FR-M55-025: B2C callback idempotent on M-Pesa reference.

## Acceptance criteria

1. AC-M55-01: 250k payroll run completes in ≤4 h at scaled load; load test at 1x local and 5x pre-prod (EC-SCL-07).
2. AC-M55-02: SoD enforced; same user cannot initiate and approve; SpiceDB policy test.
3. AC-M55-03: B2C callback idempotent; no duplicate payment; stub replay test.
4. AC-M55-04: PAYE/NSSF/NHIF calculations match statutory rates; property test with known fixtures.
5. AC-M55-05: Bank account and M-Pesa numbers not present in log output; log-scanner test.
6. AC-M55-06: Disbursement failure triggers alert after three retries; integration test.
7. AC-M55-07: API p99 latency within routine 99.5% SLA.
8. AC-M55-08: All amounts stored as integer KES; Semgrep rule passes.

## Applicable edge cases

**Scale:**
- EC-SCL-07: 250k payroll disbursements within 4 h; SLAs met at 1x and 5x.

**Identity:**
- EC-IDN-09: dual-control payroll approver revoked mid-flow; re-validate at commit.

**Data:**
- EC-DAT-01: duplicate B2C callback; single payment; idempotent refund path.
- EC-DAT-07: partial batch failure in disbursement; per-row report; valid rows committed.

**Security:**
- EC-SEC-16: bank account and M-Pesa numbers hashed in logs.
- EC-SEC-05: injection on staff ID or account fields.

**Connectivity:**
- EC-CON-05: flapping B2C API; backoff; retry budget; no duplicate.
- EC-MOB-05: push token rotation for payslip notification.

**Human:**
- EC-HUM-08: 11 languages; no truncation in application portal.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Payroll computation, SoD policy, deduction rules, B2C FSM | nextest | M55-UNIT- |
| property | Deduction correctness; disbursement idempotency; integer amounts | proptest | M55-PROP- |
| contract | M-Pesa B2C double; KRA/NSSF/NHIF doubles | Pact | M55-PACT- |
| integration | Full payroll run lifecycle; retry on B2C failure | nextest + testcontainers | M55-INT- |
| e2e | Public application portal; admin payroll run; payslip | Playwright | M55-E2E- |
| load | 250k payroll run at 1x local; 5x pre-prod | k6 | M55-LOAD- |
| security | ZAP; log scanner; SoD bypass attempt | ZAP + Semgrep | M55-SEC- |
| i18n | 11 locales | custom | M55-I18N- |
| a11y | axe on public portal and payslip | axe-core | M55-A11Y- |

Tags: `#[trace("FR-M55-010","AC-M55-01","EC-SCL-07")]`

## Runbooks and dashboards

- docs/runbooks/support-admin-svc-degradation.md (extend with payroll section)
- docs/runbooks/mpesa-b2c-disbursement-failure.md (new)
- platform/observability/support-admin-svc/payroll-dashboard.json
- platform/observability/support-admin-svc/slo-alerts.yaml (disbursement failure rate)

## Open questions

- M-Pesa B2C batch size limit (currently 1000/batch) may require chunking for 250k staff; Daraja documentation to confirm.
- NHIF/SHIF migration status affects deduction type codes; rule must be config-driven pending statutory confirmation.
