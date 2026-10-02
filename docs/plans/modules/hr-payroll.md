# M55 hr-payroll

| field | value |
|---|---|
| M-number | M55 |
| Home service | support-admin-svc (ADR-0020) |
| Owning work package | WP-5.6 (docs/plans/06-wave-5-governance.md) |
| Size | XL |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M55-hr-payroll.md |

## Purpose

Full lifecycle HR and payroll for the Commission election personnel: job applications, contracts, payroll runs (250k staff in ≤4 h), M-Pesa B2C disbursements, statutory deductions (KRA iTax, NSSF, NHIF/SHIF), and payslip distribution, with Separation of Duties for payroll authorisation.

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

Recruitment portal:

FR-M55-001: Public recruitment portal lists vacancies by role type and county; bilingual EN and SW by default (spec FR-M55-001).
FR-M55-002: Multi-step application form collects role selection, national ID, full name, phone number (M-Pesa registered), county preference, and education qualifications (spec FR-M55-001).
FR-M55-003: Phone-number OTP verification at application submission; unverified applications not accepted into the system.
FR-M55-004: Application acknowledgement SMS with unique reference number sent within 60 s of verified submission (spec FR-M55-001).
FR-M55-005: Application idempotent on national ID plus role type plus election event; second submission returns existing reference.
FR-M55-006: Public portal p99 response <= 2 s under 50k concurrent applicants during vacancy-notice period (spec NFR-M55-003).
FR-M55-007: Portal WCAG 2.2 AA compliant; keyboard-navigable; full task completion confirmed by screen reader (spec NFR-M55-004).
FR-M55-008: Portal and payslip surfaces support 11 languages; KSL explainer video available on eligibility criteria page (spec NFR-M55-005).

Eligibility screening:

FR-M55-009: Cross-check applicant national ID against M12 CRMS to detect registered candidates; conflict-flagged applications blocked from contract (spec FR-M55-002).
FR-M55-010: Cross-check against M03 ABIS for close-relative conflict per Elections Act 2011 §27; flag conflict on match (spec FR-M55-002).
FR-M55-011: NRB real-time national ID verification at application submission; unrecognised national ID returns 422 with error code.
FR-M55-012: Conflict-flagged applications blocked from contract issuance; HR Officer receives conflict queue alert with CRMS or ABIS match detail.
FR-M55-013: Conflict-flag detection must have zero false negatives; cross-check integration test uses full CRMS and ABIS doubles corpus.

Shortlisting and selection:

FR-M55-014: HR Officer scores applications against published criteria in admin console; score recorded per criterion.
FR-M55-015: System generates shortlist sorted by score per role per county (spec FR-M55-004).
FR-M55-016: Shortlist notifications sent to applicants via SMS with acceptance deadline (spec FR-M55-004).
FR-M55-017: HR Officer manual shortlist override requires mandatory written reason; override logged in audit trail.

LMS certification gate:

FR-M55-018: Before contract generation, verify applicant holds a valid M38 LMS certificate for their role type (spec FR-M55-003).
FR-M55-019: Contract generation blocked for uncertified applicants with 422; HR Officer sees certification status per applicant (spec FR-M55-003).
FR-M55-020: LMS certification heat map by county and role type displayed on HR dashboard.
FR-M55-021: LMS certificate recheck triggered automatically if certificate expires before contract start date.

Contract generation and acceptance:

FR-M55-022: Auto-generate fixed-term employment contract PDF: role, county, election event, start and end dates, daily rate, statutory deductions schedule (spec FR-M55-005).
FR-M55-023: Contract PDF digitally signed with Commission P06 HSM key; PDF hash stored on contract record (spec FR-M55-005).
FR-M55-024: Contract sent to applicant via email and SMS with acceptance link and OTP (spec FR-M55-005).
FR-M55-025: Applicant accepts contract via OTP confirmation; status set to ACCEPTED; payroll roster updated (spec FR-M55-006).
FR-M55-026: Applicant can submit wet-ink scan as alternative acceptance method; scan hash stored on contract record (spec FR-M55-006).
FR-M55-027: Contract acceptance idempotent on contract ID; second acceptance returns existing ACCEPTED status record.
FR-M55-028: Unaccepted contracts expire after 72 hours; applicant re-notified once before expiry.
FR-M55-029: Contract status lifecycle: DRAFT, SENT, ACCEPTED, DECLINED, EXPIRED, TERMINATED; each transition logged with actor and timestamp.
FR-M55-030: Bulk contract generation per role-county batch; status tracker with resend button for bounced email deliveries (spec FR-M55-005).

Payroll computation:

FR-M55-031: Payroll run initiated by HR Officer for election phase (e.g. POLLING_DAY_PLUS_2); phase selection required (spec FR-M55-009).
FR-M55-032: Compute gross pay per staff: daily_rate multiplied by days_engaged plus transport_allowance plus meal_allowance (spec FR-M55-007).
FR-M55-033: Compute KRA PAYE per casual employment tax bands loaded from config table; no hardcoded rates; update table without code deploy (spec FR-M55-008).
FR-M55-034: Compute NSSF Tier I and Tier II contributions per NSSF Act 2013 amended 2023; rates from config table (spec FR-M55-008).
FR-M55-035: Compute NHIF/SHIF contribution per current statutory rates from config table; deduction type codes config-driven pending statutory confirmation (spec FR-M55-008).
FR-M55-036: All monetary amounts computed and stored as integer KES; no floating-point arithmetic; Semgrep rule enforced in CI (spec FR-M55-021 equivalent).
FR-M55-037: Payroll run generates per-staff payroll lines with gross, PAYE, NSSF, SHIF, net amounts, M-Pesa phone, and payment status.
FR-M55-038: Payroll run computation for 250k staff completes within 4 hours on reference hardware (spec NFR-M55-001).
FR-M55-039: Payroll run preview mode: computes all amounts without committing; Finance Officer reviews totals before approval.

Payroll approval and SoD:

FR-M55-040: SoD: payroll run initiator cannot be the approver; SpiceDB policy enforced; same-user attempt returns 403 (spec NFR-M55-007).
FR-M55-041: Dual-sign payroll run approval: two Finance Officers must sign; second signer cannot be same person as first.
FR-M55-042: Approved payroll run locked from modification; further changes require rejection workflow (spec NFR-M55-008).
FR-M55-043: Rejected payroll run returned to DRAFT with mandatory rejection reason; rejection logged with rejector ID.

M-Pesa B2C disbursement:

FR-M55-044: Approved payroll run generates M-Pesa B2C payment file; submitted to Safaricom Daraja API in configurable chunk sizes (spec FR-M55-009).
FR-M55-045: 250k B2C payments submitted to Daraja API within 6 hours of Finance Officer approval (spec NFR-M55-002).
FR-M55-046: Per-staff payment status tracked: PENDING, PAID, FAILED, RERUN_PENDING (spec FR-M55-010).
FR-M55-047: Failed B2C payments (wrong number, SIM not registered) logged; HR Officer receives failed-payment list with reason codes (spec FR-M55-010).
FR-M55-048: Failed payment correction: HR Officer updates phone number; re-run executed within 48 hours of correction (spec FR-M55-010).
FR-M55-049: B2C Daraja callback idempotent on M-Pesa reference; duplicate callback produces no second payment or duplicate row.
FR-M55-050: SMS confirmation sent to each payee on successful M-Pesa receipt within 5 minutes of Daraja confirmation.
FR-M55-051: M-Pesa B2C circuit breaker: opens after 5 consecutive Daraja API failures; disbursement queued in outbox; alert emitted.
FR-M55-052: Bank account numbers and M-Pesa phone numbers stored encrypted at rest; hashed in all log output (spec FR-M55-022 equivalent).

Payslips:

FR-M55-053: Payslip generated per staff member per payroll run: gross, PAYE, NSSF, SHIF, net, and M-Pesa payment reference (spec FR-M55-013).
FR-M55-054: Payslip PDF signed with Commission P06 HSM key; hash stored on payroll line; payslip cannot be modified post-approval (spec NFR-M55-008).
FR-M55-055: Payslip accessible via staff payslip portal gated by phone number plus OTP; no cross-staff data leakage.
FR-M55-056: Payslip SMS link sent to payee within 2 hours of payroll run reaching PAID status (spec FR-M55-013).
FR-M55-057: Payslip accessible via M33 mobile app for field staff via M33 integration (spec FR-M55-017).

Statutory returns:

FR-M55-058: KRA PAYE P9 return generated per payroll run; submitted to KRA iTax API within 5 business days; confirmation reference stored (spec FR-M55-011).
FR-M55-059: NSSF contribution schedule generated per payroll run; submitted to NSSF portal API; confirmation reference stored (spec FR-M55-012).
FR-M55-060: NHIF/SHIF contribution schedule generated per payroll run; submitted to NHIF/SHIF portal API; confirmation reference stored (spec FR-M55-012).
FR-M55-061: Statutory return submission recorded with run ID, submission timestamp, and portal acknowledgement reference.
FR-M55-062: Alert on statutory return submission overdue relative to payroll run completion date; routes to eel-iems-55-alerts.

Service certificates:

FR-M55-063: Service certificate generated on demand per staff member after election completion; includes role, event, dates, and county (spec FR-M55-015).
FR-M55-064: Certificate PDF signed with Commission P06 HSM key; download gated by OTP on payslip portal (spec FR-M55-015).

By-election and referendum staffing:

FR-M55-065: Subsidiary staffing campaign activated for M49 referendum and M50 by-election events; scoped to affected constituencies (spec FR-M55-014).
FR-M55-066: Subsidiary campaign payroll fully tracked and feeds M54 subsidiary cost centre via eel.hr.payroll_run_completed.v1 event.

Budget reconciliation and reporting:

FR-M55-067: Payroll run cost totals emitted to M54 budget personnel line via eel.hr.payroll.v1 topic on run completion.
FR-M55-068: Payroll run exported as XLSX for M54 budget reconciliation; includes per-staff gross, deductions, and net.
FR-M55-069: Payroll run status dashboard for HR admin: phase, staff count, disbursement success rate, failed payment count.

Audit and data protection:

FR-M55-070: Audit trail of all payroll run state transitions with approver IDs; immutable append-only log (spec NFR-M55-008).
FR-M55-071: Payslip hash stored on payroll line; line cannot be modified post-approval; modification attempt returns 405.
FR-M55-072: Staff personal data retained 7 years per KRA and PFM Act requirements; automated purge job with alerting on overdue purge (spec NFR-M55-006).
FR-M55-073: Rejected applicant data purged 6 months after rejection decision; automated purge job with disposal log (spec NFR-M55-006).
FR-M55-074: DPIA registered with ODPC before recruitment portal goes live; Commission DPO sign-off stored as prerequisite flag.

Monitoring:

FR-M55-075: Module Grafana dashboard eel-iems-55: payroll run status by phase, B2C disbursement success rate, failed payment count, statutory returns queue depth, LMS certification rate by role.
FR-M55-076: Alert when M-Pesa B2C failure rate exceeds 2% within any single disbursement run; routes to eel-iems-55-alerts.
FR-M55-077: Alert when payroll computation duration exceeds 3 hours; early warning before 4-hour SLA breach; page emitted to on-call.

## Acceptance criteria

AC-M55-01: 250k payroll computation completes in <= 4 h at 1x reference hardware load; p99 regression <= 10% at 5x pre-prod; k6 load test (spec NFR-M55-001).
AC-M55-02: 250k M-Pesa B2C payments submitted within 6 h of Finance Officer approval; Daraja double load test confirms timing (spec NFR-M55-002).
AC-M55-03: SoD enforced: payroll run initiator cannot approve; same-user attempt returns 403; SpiceDB policy test.
AC-M55-04: Dual-sign approval enforced: second signer cannot be same person as first; attempt returns 403; unit test.
AC-M55-05: B2C Daraja callback idempotent: duplicate M-Pesa reference produces no second payment; stub replay test confirms single row.
AC-M55-06: PAYE, NSSF, and NHIF/SHIF calculations match statutory rates for 20 known fixtures including edge cases at band boundaries; property test.
AC-M55-07: Bank account numbers and M-Pesa phone numbers absent from all log output; CI log-scanner fixture passes.
AC-M55-08: All monetary amounts stored and computed as integer KES; Semgrep floating-point rule passes in CI.
AC-M55-09: Disbursement failure triggers alert after 5 consecutive B2C API failures; integration test with Daraja double returning 500.
AC-M55-10: Approved payroll run cannot be modified; modification attempt returns 405; unit test.
AC-M55-11: Conflict-flag detection zero false negatives on CRMS plus ABIS double corpus of 10k known conflicts; integration test.
AC-M55-12: LMS certification gate blocks contract generation for uncertified applicant; attempt returns 422; Pact contract test against M38 double.
AC-M55-13: Public recruitment portal p99 <= 2 s under 50k concurrent applicants; k6 load test confirms (spec NFR-M55-003).
AC-M55-14: Staff payslip endpoint returns 403 for any staff ID other than the authenticated caller; cross-staff access security test.
AC-M55-15: KRA PAYE P9 return submitted to iTax within 5 business days of payroll run; scheduler integration test confirms timing.
AC-M55-16: WCAG 2.2 AA on public portal and payslip portal; axe-core zero critical violations; screen-reader full-task-completion test passes (spec NFR-M55-004).
AC-M55-17: 11 languages on public application portal with no truncation; 150% pseudo-locale test and all 11-locale snapshots pass (spec NFR-M55-005).
AC-M55-18: Rejected applicant data purged after 6-month retention period; automated purge job tested with synthetic aged records (spec NFR-M55-006).
AC-M55-19: Failed-payment rate < 2% in 250k-disbursement load test with Daraja double calibrated to Kenya-typical failure scenarios.
AC-M55-20: Module availability >= 99.5% during active election period; synthetic monitor confirms uptime from nomination close to T+30 (spec NFR-M55-009).

## Applicable edge cases

EC-DAT-01: Duplicate B2C Daraja callback with same M-Pesa reference; idempotent; single payment recorded; no duplicate payroll line created.
EC-DAT-02: Same payroll run idempotency key submitted with different staff count; 422 idempotency-key-reuse returned; no second run created.
EC-DAT-06: Payroll event schema v1 and v2 coexist during rolling deploy; both accepted; dual-emit; Karapace compatibility check passes.
EC-DAT-07: Partial B2C batch failure (some staff payments fail); per-staff status report generated; successful payments committed; failed list populated for re-run.
EC-DAT-08: 250k-row payroll computation; streaming pipeline with checkpoint-resumable progress; bounded memory; kill-resume integration test confirms no duplicate lines.
EC-DAT-09: Staff names contain Unicode characters (Swahili diacritics, Arabic script for Somali staff); NFC normalised; grapheme-safe; 11-locale snapshot tests pass.
EC-SEC-05: SQL injection on staff ID, national ID hash, or M-Pesa phone number field; parameterised queries block; Semgrep compile-time check and ZAP scan both pass in CI.
EC-SEC-16: Bank account numbers and M-Pesa phone numbers hashed in all log lines and OpenTelemetry trace attributes; log-scanner CI fixture passes.
EC-IDN-02: JWT expires mid-payroll-run-approval request; in-flight request completes; next request returns 401 with silent-refresh hint in WWW-Authenticate header.
EC-IDN-07: SIM swap on staff member phone for payslip OTP access; device-bound OTP check; new device denied without re-enrolment via HR Officer.
EC-IDN-10: SpiceDB unavailable during payroll run approval; fail closed; 503 problem+json returned; no partial approval committed to database.
EC-SCL-04: Consumer lag on eel.hr.payroll.v1 topic exceeds 60 s; alert paged; backpressure applied to producers; no OOM on consumer pod.
EC-SCL-07: 250k payroll disbursements at 1x load; SLAs met; p99 regression <= 10% at 5x pre-prod; k6 load test with Daraja double simulator.
EC-CON-05: Flapping M-Pesa B2C API; exponential backoff with jitter; retry budget enforced; no duplicate payment submitted to Daraja.
EC-CON-11: Kafka ISR shrink during payroll event publish; producer blocks; outbox accumulates; alert paged; no event loss confirmed after ISR recovers.
EC-MOB-05: FCM push token rotation for payslip notification; idempotent re-register; SMS used as fallback for critical pay confirmation when FCM unavailable.
EC-HUM-02: Staff applicant with missing finger (amputee); biometric exception path at contract collection; face plus ID alternative; no denial of application.
EC-HUM-08: 11 languages on public application portal including long Swahili and Somali strings; no truncation; ICU plurals applied; font fallback stack confirmed.
EC-OPS-07: Staff officer no-show on polling day; reassignment workflow; credential issued exactly once to replacement; original credential revoked.
EC-LEG-10: Duplicate Daraja payment callback arrives after manual re-run; single payment credited; idempotent; no double-count in payroll totals.
EC-TIM-09: Payroll computation race condition at payroll freeze deadline; OPA hard-close at transaction time; all-or-nothing commit; concurrent writer test.
EC-SEC-04: KRA tax rate config KEK rotation mid-election cycle; online re-wrap; old config readable until rotation complete; full audit trail of re-wrap.
EC-OPS-01: Server power outage mid-payroll-computation; SQLite WAL resume on restart; no duplicate payroll line created; journal-before-act invariant.
EC-DAT-05: Poison payroll message on eel.hr.payroll.v1 topic (staff count exceeds total registered election workforce); DLQ with reason; Kafka partition continues processing.
EC-ML-05: PII in KRA iTax return log or iTax API request trace; PIIStrippingEvent fired before external API call; regression corpus and Semgrep rule both pass.
EC-SEC-09: Rate-limit evasion on payslip endpoint; key includes staff ID plus /64 subnet; priority shedding enforced; k6 rotating-IP test passes.

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
