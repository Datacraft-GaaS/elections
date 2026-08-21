# M47 nomination-fee-payment

| field | value |
|---|---|
| M-number | M47 |
| Home service | crms-svc (ADR-0020) |
| Owning work package | WP-4.11 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M47-nomination-fee-payment.md |

## Purpose

Manages nomination fee schedules, M-Pesa C2B payment collection, waivers, and refunds for candidates registering through M12. Ensures idempotent payment processing via Daraja callbacks and supports reconciliation and audit reporting.

## Surfaces and languages

| surface | technology | path under services/crms-svc/ |
|---|---|---|
| Public fee payment portal | Astro + Django | astro/crms-public/, django/crms-admin/ |
| Candidate self-service | Django (existing crms-admin) | django/crms-admin/ |
| Admin reconciliation console | Django | django/crms-admin/ |
| Daraja webhook receiver | Python FastAPI | py/daraja/ |
| Internal API | Rust (axum) | crates/crms-api/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| FeeSchedule | election_id, category (presidential/gubernatorial/senator/women-rep/mp/ward), amount_kes, effective_from | C | T1-consensus |
| Payment | id (UUIDv7), candidate_id, fee_schedule_id, mpesa_reference, status (pending/confirmed/failed/refunded), amount_kes, callback_received_at | B | T1-consensus |
| Waiver | id, candidate_id, grounds (gender/disability/youth), evidence_url, approver_id, granted_at | B | T1-audit |
| Refund | id, payment_id, reason, amount_kes, mpesa_reference, status, processed_at | B | T1-consensus |
| ReconciliationReport | election_id, generated_at, total_collected_kes, total_waivers, total_refunds, shortfall | D | T2 |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/fees/schedules | 1 signed-static | public | n/a | current fee schedule per election |
| POST /v1/fees/payments/initiate | 2 live-per-user | JWT | yes (Idempotency-Key) | initiates STK push |
| POST /v1/fees/payments/callback | internal gRPC | SVID mTLS | yes (mpesa_reference) | Daraja webhook; idempotent on mpesa_reference |
| GET /v1/fees/payments/{id} | 2 live-per-user | JWT | n/a | payment status |
| POST /v1/fees/waivers | 3 accredited | JWT+SpiceDB | yes | RO submits waiver application |
| PUT /v1/fees/waivers/{id}/decide | 3 accredited | JWT+SpiceDB | yes | approve/reject with reason |
| POST /v1/fees/refunds | 3 accredited | JWT+SpiceDB | yes | initiate refund |
| GET /v1/fees/reconciliation | 3 accredited | JWT+SpiceDB | n/a | summary report |
| GET /v1/fees/reconciliation/export | 4 bulk | JWT+SpiceDB | n/a | CSV/XLSX export |

OpenAPI: packages/openapi-contracts/crms/nomination-fee-payment.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.crms.payment_confirmed.v1 | emits | eel.crms.payment.v1 | operational | no | at-least-once idempotent |
| eel.crms.payment_failed.v1 | emits | eel.crms.payment.v1 | operational | no | at-least-once idempotent |
| eel.crms.waiver_granted.v1 | emits | eel.crms.waiver.v1 | operational | no | at-least-once idempotent |
| eel.crms.refund_processed.v1 | emits | eel.crms.payment.v1 | operational | no | at-least-once idempotent |
| eel.crms.nomination_submitted.v1 | consumes | eel.crms.nomination.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M-Pesa Daraja C2B | mpesa-daraja | integrations/mpesa-daraja/ | yes | circuit breaker; payment stays pending, retry on reconnect |
| M-Pesa Daraja B2C (refunds) | mpesa-daraja | integrations/mpesa-daraja/ | yes | circuit breaker; refund queued, alerting |
| M12 CRMS nomination | internal | crates/crms-api/ | yes | hard dependency; fail if candidate not found |
| M45 party registration | internal | crates/crms-api/ | yes | advisory; party check on candidate |

## ML and AI

none

## Dependencies

- Modules: M12 (candidate record), M45 (party status check)
- Platform: P01 (SpiceDB authz), P04 (Kafka outbox), P05 (transparency log for audit events)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs, eel-web-py (Django admin)

## Feature checklist

1. FR-M47-001: Display current fee schedule per election and candidate category on public portal.
2. FR-M47-002: Initiate STK push via Daraja C2B with candidate mobile number and amount.
3. FR-M47-003: Receive Daraja C2B callback and confirm payment idempotently on mpesa_reference.
4. FR-M47-004: Reject duplicate Daraja callback with same mpesa_reference and different amount (422).
5. FR-M47-005: Return payment status on GET /payments/{id}.
6. FR-M47-006: Block nomination submission in M12 when payment status is not confirmed or waiver not granted.
7. FR-M47-007: Allow RO to submit waiver application with documentary evidence URL.
8. FR-M47-008: Route waiver for dual approval (RO + Commissioner) above threshold amount.
9. FR-M47-009: Approve or reject waiver with mandatory reason; emit waiver event.
10. FR-M47-010: Record waiver grounds (gender, disability, youth) with evidence reference.
11. FR-M47-011: Initiate B2C refund on nomination withdrawal or disqualification.
12. FR-M47-012: Track refund status through Daraja B2C callback.
13. FR-M47-013: Alert operations team when refund fails after three retries.
14. FR-M47-014: Generate daily reconciliation report (collected, waivers, refunds, shortfall).
15. FR-M47-015: Export reconciliation as CSV and XLSX with audit timestamp.
16. FR-M47-016: Admin screen lists all payments with filter by election, category, status.
17. FR-M47-017: Admin screen shows audit trail of payment status transitions.
18. FR-M47-018: Admin can manually override payment status with dual approval and reason (break-glass).
19. FR-M47-019: Fee schedule management: create, publish, retire per election.
20. FR-M47-020: Fee schedule is versioned and signed; changes require approval.
21. FR-M47-021: System enforces that only one active schedule exists per category per election.
22. FR-M47-022: Candidate can view their payment receipt with mpesa reference and timestamp.
23. FR-M47-023: Candidate can download PDF receipt for cleared payment.
24. FR-M47-024: All financial amounts stored in KES integer (no floating point).
25. FR-M47-025: System enforces Idempotency-Key header on all payment initiation requests.
26. FR-M47-026: Payment initiation returns pending status if STK push succeeds; failed if telco rejects.
27. FR-M47-027: STK push times out after 60 s; status set to failed automatically.
28. FR-M47-028: i18n: all portal text in 11 languages; Swahili mandatory.
29. FR-M47-029: WCAG 2.2 AA compliance on all public surfaces.
30. FR-M47-030: Audit log records every payment state transition with actor and timestamp.

## Acceptance criteria

1. AC-M47-01: Duplicate Daraja callback with same mpesa_reference returns 200 with cached result and triggers no second DB write (EC-DAT-01, EC-LEG-10).
2. AC-M47-02: Duplicate callback with same key but different amount returns 422 idempotency-key-reuse (EC-DAT-02).
3. AC-M47-03: Payment confirmed event emitted exactly once per confirmed payment; consumer idempotency test passes.
4. AC-M47-04: Nomination in M12 is blocked until payment confirmed or waiver granted; e2e test proves.
5. AC-M47-05: Waiver approval requires dual-officer sign-off above configurable threshold; unit test.
6. AC-M47-06: Refund B2C callback idempotent; refund_processed event emitted once; test.
7. AC-M47-07: Reconciliation export matches sum of payment rows; property test on 10k synthetic rows.
8. AC-M47-08: API p99 latency under nominal load at routine 99.5% SLA.
9. AC-M47-09: All amounts stored as integer KES; no floating-point arithmetic in payment path; Semgrep rule passes.
10. AC-M47-10: 11-locale snapshot tests pass; no hardcoded strings.

## Applicable edge cases

**Data integrity:**
- EC-DAT-01: duplicate payment callback with same key replayed; must cache and return without re-executing.
- EC-DAT-02: same Idempotency-Key with different body; 422 returned.
- EC-DAT-10: zero-balance waiver (full fee waived); no divide-by-zero in reconciliation.

**Legal:**
- EC-LEG-10: duplicate Daraja callback; single payment recorded, idempotent refund.

**Security:**
- EC-SEC-05: injection on fee amount or mpesa reference fields; parameterised queries only.
- EC-SEC-09: rate-limit evasion on payment initiation; key by subject and IP /64.
- EC-SEC-16: payment amounts and mpesa references must not appear in logs.

**Scale:**
- EC-SCL-08: Valkey eviction loses idempotency key; PG-durable idempotency store.

**Human:**
- EC-HUM-08: 11 languages including long strings; no truncation in UI.

**Connectivity:**
- EC-CON-05: flapping Daraja webhook; backoff with retry budget; no duplicate.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | FeeSchedule validation, payment FSM, waiver threshold logic, refund state machine | nextest | M47-UNIT- |
| property | Reconciliation sum invariant; duplicate callback idempotency; amount integer enforcement | proptest | M47-PROP- |
| contract | Daraja C2B/B2C doubles; M12 nomination block | Pact | M47-PACT- |
| integration | Full payment flow with testcontainers; waiver dual-approval flow | nextest + testcontainers | M47-INT- |
| e2e | Public portal payment → nomination unblock in M12 | Playwright | M47-E2E- |
| fuzz | Daraja callback payload fields; fee amount edge cases | cargo-fuzz | M47-FUZZ- |
| security | OWASP ZAP on public endpoints; Semgrep injection rules; log scanner | ZAP + Semgrep | M47-SEC- |
| load | Payment initiation at peak nomination period | k6 | M47-LOAD- |
| i18n | 11 locales + 150% pseudo-locale on portal | custom + Playwright | M47-I18N- |
| a11y | axe on public portal and admin screens | axe-core + Playwright | M47-A11Y- |

Tags: `#[trace("FR-M47-001","AC-M47-01","EC-LEG-10")]`

## Runbooks and dashboards

- docs/runbooks/crms-svc-degradation.md (extend with payment-callback failure mode)
- docs/runbooks/mpesa-b2c-refund-failure.md (new)
- platform/observability/crms-svc/nomination-fee-payment-dashboard.json
- platform/observability/crms-svc/slo-alerts.yaml (p99 latency, callback failure rate)

## Open questions

- Daraja sandbox approval timeline for IEBC merchant account; double must gate development until live credentials arrive.
- Legal confirmation on waiver grounds and required evidence types under Elections Act s.23.
- Whether partial waivers (discounted fee) are in scope or only full waivers.
