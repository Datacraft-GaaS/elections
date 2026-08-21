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

Fee schedule:
1. FR-M47-001: Fee schedule configured per contest tier (President, Governor, Senator, MP, CWR, MCA) per IEBC gazette notice; schedule version-controlled; active schedule drives all payment validations.
2. FR-M47-002: Fee schedule published on public portal (no login required) with gazette reference and effective date.
3. FR-M47-019: Fee schedule management: create, publish, retire per election; changes require approval before activation.
4. FR-M47-020: Fee schedule is versioned and HSM-signed; only one active schedule per category per election enforced.
5. FR-M47-021: System enforces that only one active schedule exists per category per election; duplicate activation attempt returns 409.

M-Pesa payment:
6. FR-M47-003: M-Pesa Paybill payment: portal generates unique UUID7-based payment reference; candidate pays via Paybill; Daraja C2B callback confirms payment; reference matched to candidate in M12.
7. FR-M47-025: Idempotency-Key header enforced on all payment initiation requests; missing header returns 400.
8. FR-M47-026: Payment initiation returns PENDING status if STK push accepted by Daraja; FAILED if telco rejects immediately.
9. FR-M47-027: STK push times out after 60 s; status transitions to FAILED automatically; candidate notified.
10. FR-M47-004: Duplicate Daraja C2B callback with same mpesa_reference and different amount returns 422 idempotency-key-reuse; original payment unchanged.
11. FR-M47-005: Payment status returned on GET /v1/fees/payments/{id}; includes mpesa_reference, confirmed_at, and amount_kes.

Bank transfer payment:
12. FR-M47-031: Bank transfer payment: portal generates unique pay-in reference; candidate presents to bank; Finance Officer manually confirms receipt within 2 working days; candidate notified on confirmation.
13. FR-M47-033: Payment reference expiry: M-Pesa reference expires 24 hours after generation; bank transfer reference expires 5 working days; expired references return EXPIRED status; candidate notified.

Party bulk payment:
14. FR-M47-032: Party bulk payment: party official generates batch of payment references for multiple candidates; pays single bank transfer; Finance Officer splits and allocates per candidate on confirmation.

Nomination gate:
15. FR-M47-006: M12 nomination workflow blocked until payment status is CONFIRMED or WAIVED; unpaid candidates cannot advance to DOCS_UPLOADED step; M12 queries M47 API for current status.
16. FR-M47-009: Payment window enforcement: references generated only within the gazetted nomination period; late requests rejected with statutory reference.

Waiver:
17. FR-M47-007: Candidate submits waiver application with grounds (financial hardship, disability, youth per IEBC Regulations) and supporting document URLs; evidence format-validated.
18. FR-M47-008: Waiver routed for dual approval (Finance Officer + Commissioner) above configurable threshold amount.
19. FR-M47-010: Waiver grounds (gender, disability, youth) recorded with evidence reference; grounds immutable after submission.
20. FR-M47-034: Waiver decision issued within 5 working days; decision letter (signed PDF via P06 HSM) sent to candidate; WAIVED status propagated to M12.
21. FR-M47-035: Waiver anonymised statistics: IEBC publishes count by contest tier and county post-nomination window; no individual names published.

Refund:
22. FR-M47-011: B2C refund initiated on nomination disqualification (full refund, Elections Act s.22(4)) or withdrawal before gazette (50% refund per IEBC Regulations); refund basis recorded.
23. FR-M47-012: Refund status tracked through Daraja B2C callback; PENDING -> APPROVED -> PAID state machine.
24. FR-M47-013: Alert operations team when refund fails after three retries; escalation recorded in audit log.
25. FR-M47-036: Refund PDF receipt: digitally signed via P06 HSM; downloadable from candidate portal on PAID status.
26. FR-M47-037: Refund reconciliation: Finance Officer reconciles all refunds against IEBC designated account by T+30; reconciliation report archived with Merkle path.

Receipts and audit:
27. FR-M47-022: Candidate views payment receipt with mpesa_reference, confirmed_at, and amount_kes on portal.
28. FR-M47-023: Candidate downloads HSM-signed PDF receipt on CONFIRMED status; receipt generated within 30 seconds of CONFIRMED event.
29. FR-M47-024: All financial amounts stored as KES integer (no floating point); Semgrep rule enforced in CI.
30. FR-M47-030: Audit log records every payment state transition with actor, timestamp, and reason; appended to P05 Merkle chain.

Reconciliation:
31. FR-M47-014: Daily reconciliation report: collected, waivers, refunds, shortfall; available to Finance Officer and Auditor.
32. FR-M47-015: Export reconciliation as CSV and XLSX with audit timestamp; export includes P05 Merkle path for each payment event.
33. FR-M47-016: Admin payment queue: filter by election, category, status; unmatched items flagged.
34. FR-M47-017: Admin audit trail view: all payment status transitions with actor and timestamp.
35. FR-M47-018: Admin manual payment status override with dual approval and reason (break-glass); override event logged to P05.

Portal and accessibility:
36. FR-M47-028: i18n: all portal text in 11 languages; Swahili mandatory; no hardcoded strings; ICU plurals.
37. FR-M47-029: WCAG 2.2 AA compliance on all public and candidate-facing surfaces; EN + SW.

## Acceptance criteria

1. AC-M47-01: Duplicate Daraja callback with same mpesa_reference returns 200 with cached result and triggers no second DB write; idempotency integration test passes (EC-DAT-01, EC-LEG-10).
2. AC-M47-02: Duplicate callback with same mpesa_reference but different amount returns 422 idempotency-key-reuse; original payment unchanged; test passes (EC-DAT-02).
3. AC-M47-03: eel.crms.payment_confirmed.v1 event emitted exactly once per confirmed payment; consumer idempotency test and property test pass.
4. AC-M47-04: Nomination in M12 blocked until payment CONFIRMED or waiver WAIVED; e2e Playwright test confirms block and unblock.
5. AC-M47-05: Waiver approval requires dual-officer sign-off above configurable threshold; unit test validates both approver signatures required.
6. AC-M47-06: Refund B2C callback idempotent; eel.crms.refund_processed.v1 emitted once; Daraja B2C stub replay test.
7. AC-M47-07: Reconciliation export total collected matches sum of CONFIRMED payment rows; property test on 10k synthetic rows with zero shortfall.
8. AC-M47-08: API p99 latency <= defined threshold under 2,000 concurrent users during nomination window; k6 load test at cycle-critical 99.9% availability class.
9. AC-M47-09: All amounts stored as KES integer; no floating-point arithmetic in payment or refund path; Semgrep rule `no-float-kes` passes in CI.
10. AC-M47-10: 11-locale snapshot tests pass; no hardcoded strings; 150% pseudo-locale renders without truncation.
11. AC-M47-11: M-Pesa STK push confirmation end-to-end <= 5 s; Daraja sandbox integration test measures round-trip; circuit breaker fires after 3 consecutive failures.
12. AC-M47-12: Bank transfer reference expires at 5 working days; status transitions to EXPIRED automatically; scheduler unit test with mocked clock passes.
13. AC-M47-13: System availability >= 99.9% during nomination window (T-60 to T-46); SLO burn-rate alert fires at > 0.1% error budget per hour; Alertmanager test passes.
14. AC-M47-14: Payment data encrypted at rest; Daraja phone number stored only for refund routing; 7-year retention enforced; DPIA registration confirmed; DPA compliance audit passes.

## Applicable edge cases

data:
- EC-DAT-01: duplicate Daraja C2B callback with same mpesa_reference replayed; cached 200 returned; no second DB write; no duplicate payment created
- EC-DAT-02: same Idempotency-Key with different payment body; 422 idempotency-key-reuse returned; original payment unchanged
- EC-DAT-05: waiver amount exceeds fee schedule amount (poison waiver); validation rejects; Finance Officer alerted; no waiver persisted
- EC-DAT-07: partial batch failure in party bulk payment allocation; per-candidate report returned; successfully allocated candidates unaffected; failed allocations retryable
- EC-DAT-10: zero-balance waiver (full fee waived); no divide-by-zero in reconciliation; shortfall computed as 0; property test passes
- EC-DAT-14: oversized Daraja callback payload (> 1 MB); outbox rejects with 413; logged; callback not silently dropped

legal:
- EC-LEG-02: court-ordered suspension of nomination window; payment reference generation blocked; existing references retain original expiry; candidates notified via portal
- EC-LEG-10: duplicate Daraja callback; single payment recorded; refund idempotent; eel.crms.payment_confirmed.v1 emitted once

security:
- EC-SEC-03: P06 HSM unavailable during receipt generation; generation deferred to HSM recovery; candidate notified receipt delayed; no corrupt receipt issued
- EC-SEC-05: injection on fee_amount, mpesa_reference, or candidate_id fields; parameterised queries only; Semgrep and ZAP tests pass
- EC-SEC-06: SSRF via receipt URL or evidence document URL; allowlist enforced; private-range redirect blocked; Nuclei test passes
- EC-SEC-09: rate-limit evasion on payment initiation; key by subject and IP/64; k6 rotating IP test passes
- EC-SEC-11: timing side-channel on payment reference lookup; constant-time comparison; dudect-style test passes
- EC-SEC-16: payment amounts and mpesa_reference must not appear in logs, traces, or spans; log-scanner CI fixture passes on every build

identity:
- EC-IDN-02: candidate JWT expires mid-payment wizard; in-flight step completes; next request returns 401 and silent refresh; no payment data lost
- EC-IDN-09: waiver dual-approval; approver revoked mid-flow; re-validation at commit; approval blocked if approver revoked; compensating event logged
- EC-IDN-10: SpiceDB down during waiver approval action; fail closed; 503 returned; no partial approval persisted

scale:
- EC-SCL-01: 2,000 concurrent STK push initiations during nomination-window open; STK queue handles burst; p99 <= 5 s end-to-end; k6 load test
- EC-SCL-04: Kafka consumer lag > 60 s during nomination peak; backpressure applied; no payment event lost; SRE alerted
- EC-SCL-08: Valkey eviction loses idempotency key; PG-durable idempotency store fallback; no duplicate payment recorded

connectivity:
- EC-CON-05: flapping Daraja C2B webhook; exponential backoff with retry budget; no duplicate payment confirmed; server dedupe count 0

operations:
- EC-OPS-05: Daraja API downtime during nomination window; bank transfer fallback pathway activated; Finance Officer manual confirmation within 2 working days; circuit breaker opens after 3 consecutive Daraja failures

human:
- EC-HUM-08: 11 languages including long strings; no truncation on payment portal; 150% pseudo-locale test passes
- EC-HUM-06: screen reader support on payment portal; full task completion via keyboard and ARIA; axe-core audit passes

time:
- EC-TIM-04: payment reference expiry computed in EAT; nomination window boundaries in EAT; UTC storage with EAT display; boundary test passes

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
