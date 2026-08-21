# M22 campaign-finance-transparency

| field | value |
|---|---|
| M-number | M22 |
| Home service | transparency-svc (ADR-0020) |
| Owning work package | WP-4.6 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M22-campaign-finance-transparency.md |

## Purpose

Statutory campaign finance disclosure: accept contribution and expenditure disclosures from candidates and parties, cross-reference with CBK/FRC transaction data, flag anomalies, publish aggregated public reports, and generate evidence packages for IEBC and financial regulators.

## Surfaces and languages

| surface | technology | path under services/transparency-svc/ |
|---|---|---|
| Party/candidate disclosure portal | Astro on packages/eel-web-astro | astro/finance-portal/ |
| Disclosure API | Rust (axum + utoipa) | crates/finance-api/ |
| Anomaly detection worker | Rust | crates/finance-anomaly/ |
| Admin back-office | Django on packages/eel-web-py | django/finance-admin/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| CandidateDisclosure | disclosure_id, candidate_id, election_id, period, contributions_kes, expenditures_kes, submitted_at, sig | C | T2 |
| CashAggregate | agg_id, candidate_id, period, total_in, total_out, ceiling_kes, status, computed_at | D | T2 |
| AnomalyFlag | flag_id, disclosure_id, reason, severity, cbk_ref, status, resolved_at | C | T2 |
| EvidencePackage | pkg_id, candidate_id, election_id, content_refs, chain_of_custody, sig | C | T1-audit |
| AuditEvent | event_id, actor, action, ts_hlc, sig | C | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/transparency/finance.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /finance/v1/disclosures | 2 live-per-user | JWT | yes | Submit disclosure for a period |
| GET /finance/v1/disclosures/{id} | 2 live-per-user | JWT | n/a | Disclosure status |
| GET /finance/v1/candidates/{id}/summary | 1 signed-static | public | n/a | Public aggregate for a candidate (ceiling status) |
| GET /finance/v1/reports | 1 signed-static | public | n/a | Published election-cycle finance report |
| GET /finance/v1/flags | 3 accredited | JWT+SpiceDB | n/a | Anomaly flag feed (IEBC internal) |
| PATCH /finance/v1/flags/{id} | 3 accredited | JWT+SpiceDB | yes | Adjudicate anomaly flag |
| POST /finance/v1/evidence/{candidate_id} | 3 accredited | JWT+SpiceDB | yes | Assemble evidence package |
| GET /finance/v1/evidence/{id} | 3 accredited | JWT+SpiceDB | n/a | Evidence package for regulator |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.finance.disclosure_submitted.v1 | emits | eel.finance.disclosure.v1 | operational | yes | at-least-once idempotent |
| eel.finance.anomaly_flagged.v1 | emits | eel.finance.anomaly.v1 | operational | yes | at-least-once idempotent |
| eel.finance.evidence_assembled.v1 | emits | eel.finance.evidence.v1 | operational | yes (HSM) | exactly-once |
| eel.crms.candidate_approved.v1 | consumes | eel.crms.candidate.v1 | operational | yes | at-least-once idempotent |
| eel.grievance.case_created.v1 | consumes | eel.grievance.case.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| CBK/FRC | external | integrations/cbk/ | yes (stub) | async cross-reference; queue; alert on fail |
| CRMS M12 | internal | integrations/crms/ | yes | candidate identity verify |
| Grievance channels M23 | internal | integrations/grievance/ | yes | finance-complaint signal |
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed on evidence package sign |
| Transparency log P05 | internal | integrations/tlog/ | yes | report + evidence anchoring |

## ML and AI

Anomaly detection: rule-based ceiling check (Elections Act s.98) + statistical outlier detection on disclosure amounts. No GPU required. Model card not required (rule-based only).

## Dependencies

- Modules: M12 (candidate identity), M23 (finance complaint source)
- Platform: P05 (audit), P06 (HSM)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-web-astro, eel-web-py

## Feature checklist

1. FR-M22-001: periodic disclosure submission: contributions and expenditures per reporting period.
2. FR-M22-002: spending ceiling enforcement: Elections Act s.98 limits per position; flag if exceeded.
3. FR-M22-003: CBK/FRC transaction cross-reference: compare disclosed amounts against banking data.
4. FR-M22-004: anomaly detection: ceiling breach, undisclosed income, round-number clustering, rapid drawdown.
5. FR-M22-005: anomaly adjudication workflow: IEBC officer reviews; accepts, dismisses, or escalates.
6. FR-M22-006: public aggregate report: per-candidate ceiling status; no raw transaction detail published.
7. FR-M22-007: evidence package assembly: HSM-signed; chain of custody; sent to IEBC and FRC.
8. FR-M22-008: statutory report: post-election campaign finance cycle report; published and anchored.
9. FR-M22-009: grievance intake: finance-related complaints from M23 routed to anomaly queue.
10. FR-M22-010: RBAC: candidate portal, party treasurer, IEBC finance desk, FRC access.
11. FR-M22-011: DPA compliance: raw banking data not retained beyond cross-reference window.
12. FR-M22-012: audit trail: every disclosure submission and adjudication signed.
13. Ceiling check runs within 60 s of disclosure submission.
14. CBK cross-reference async; result within 24 h; alert if > 48 h.
15. Public aggregate report updated within 1 h of each disclosure period close.
16. Evidence package assembled within 30 min of request.

## Acceptance criteria

AC-M22-01: ceiling breach detected and flagged within 60 s of submission (integration test).
AC-M22-02: CBK cross-reference discrepancy surfaced as anomaly flag (integration test).
AC-M22-03: evidence package chain of custody verifiable (integration test).
AC-M22-04: raw banking data purged after cross-reference window (data retention audit).
AC-M22-05: public aggregate shows ceiling status without raw transaction detail (security review).
AC-M22-06: 99.9% availability during campaign finance reporting cycle (SLO measurement).

## Applicable edge cases

**time**
EC-TIM-09: reporting deadline OPA hard-close; in-flight disclosures must complete.
EC-TIM-04: CBK SLA breach > 48 h; manual cross-reference path; IEBC finance desk notified.

**identity**
EC-IDN-02: JWT expires mid-disclosure; in-flight completes; next 401.
EC-IDN-10: SpiceDB down; fail closed anomaly adjudication writes.

**data**
EC-DAT-01: duplicate disclosure same key; cached.
EC-DAT-09: Unicode candidate names; NFC.
EC-DAT-17: CBK data format change; adapter version-negotiation; alert.

**security**
EC-SEC-05: injection on disclosure form; parameterised.
EC-SEC-16: secrets in disclosure fields; PII-strip after cross-reference.
EC-SEC-06: SSRF via supporting document URL; allowlist.

**legal**
EC-LEG-10: FRC MoU must be in place before CBK data sharing begins; legal must confirm before G4.
EC-LEG-11: Elections Act s.98 ceiling interpretation contested; IEBC legal confirms computation method before implementation.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | ceiling check, anomaly rules, CBK cross-ref parser, evidence builder, purge scheduler | nextest | M22-UNIT- |
| property | ceiling breach always flagged; evidence hash stable | proptest | M22-PROP- |
| contract | M12 candidate identity (consumer); M23 grievance signal (consumer) | Pact | M22-PACT- |
| integration | CBK stub; M12 double; SoftHSM evidence sign; tlog anchor | nextest + testcontainers | M22-INT- |
| e2e | disclosure submit -> ceiling check -> CBK cross-ref -> anomaly flag -> adjudicate -> evidence | Playwright | M22-E2E- |
| security | injection on disclosure; SSRF; PII-strip regression; OWASP ZAP | ZAP + Semgrep | M22-SEC- |
| load | campaign finance period close; concurrent disclosures | k6 | M22-LOAD- |

## Runbooks and dashboards

- docs/runbooks/finance-transparency-degradation.md (create): CBK unavailable; cross-reference lag; HSM fail; FRC handoff failure.
- platform/observability/transparency-svc/grafana-finance.json: disclosure rate, ceiling breach count, CBK cross-ref lag, anomaly flag queue depth.
- platform/observability/transparency-svc/slo-alerts.yaml: 99.9% burn rate; CBK lag > 48 h; ceiling breach unacknowledged > 4 h.

## Open questions

- CBK/FRC MoU: the data-sharing agreement has not been signed; legal must confirm scope, data minimisation requirements, and liability before CBK cross-reference is enabled.
- Ceiling computation method: the Elections Act s.98 does not specify exactly how in-kind contributions are valued; IEBC legal must confirm the valuation method before the ceiling check is implemented.
