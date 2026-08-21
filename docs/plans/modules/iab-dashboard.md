# M15 iab-dashboard

| field | value |
|---|---|
| M-number | M15 |
| Home service | ecc-svc (ADR-0020) |
| Owning work package | WP-4.1 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M15-iab-dashboard.md |

## Purpose

Independent Audit Board dashboard: give IAB members a real-time, hardware-key-authenticated window into election operations data (reproducibility bundles, inventory status, anomaly flags, complaints), support escalation workflows to IEBC leadership, and produce statutory compliance reports.

## Surfaces and languages

| surface | technology | path under services/ecc-svc/ |
|---|---|---|
| IAB dashboard admin | Elixir (Phoenix LiveView) | ex/iab-live/ |
| IAB API | Rust (axum + utoipa) | crates/iab-api/ |
| Report generator | Rust + Typst | crates/iab-reports/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| IABEscalation | escalation_id, raised_by, category, description, evidence_refs, status, resolved_at | C | T2 |
| Complaint | complaint_id, source_module, complaint_text, severity, status, escalation_id | C | T2 |
| Report | report_id, type, period, content_hash, sig, published_at | D | T1-audit |
| AuditEvent | event_id, actor, action, ts_hlc, sig | C | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/iab/dashboard.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /iab/v1/summary | 3 accredited | JWT+SpiceDB+HW-key | n/a | Live election summary for IAB members |
| GET /iab/v1/bundle/{id} | 3 accredited | JWT+SpiceDB+HW-key | n/a | Reproducibility bundle status from M14 |
| GET /iab/v1/inventory | 3 accredited | JWT+SpiceDB+HW-key | n/a | Inventory status from M20 |
| GET /iab/v1/anomalies | 3 accredited | JWT+SpiceDB+HW-key | n/a | Anomaly flags from M26 |
| POST /iab/v1/escalations | 3 accredited | JWT+SpiceDB+HW-key | yes | Raise escalation |
| PATCH /iab/v1/escalations/{id} | 3 accredited | JWT+SpiceDB+HW-key | yes | Update escalation status |
| POST /iab/v1/reports | 3 accredited | JWT+SpiceDB+HW-key | yes | Publish signed compliance report |
| GET /iab/v1/reports | 3 accredited | JWT+SpiceDB+HW-key | n/a | List published reports |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.iab.escalation_raised.v1 | emits | eel.iab.escalation.v1 | operational | yes | at-least-once idempotent |
| eel.iab.escalation_resolved.v1 | emits | eel.iab.escalation.v1 | operational | yes | at-least-once idempotent |
| eel.iab.report_published.v1 | emits | eel.iab.report.v1 | operational | yes (HSM) | exactly-once |
| eel.transparency.bundle_published.v1 | consumes | eel.transparency.bundle.v1 | operational | yes | exactly-once |
| eel.inventory.asset_status_changed.v1 | consumes | eel.inventory.asset.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| Transparency bundle M14 | internal | integrations/reproducibility/ | yes | cached status; stale labelled |
| Inventory M20 | internal | integrations/inventory/ | yes | cached; stale labelled |
| Anomaly/fraud M26 | internal | integrations/anomaly/ | yes | feed polled; circuit breaker |
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed on report publish |
| Transparency log (P05) | internal | integrations/tlog/ | yes | report anchoring queued |
| Hardware key (P06/FIDO2) | I07 | integrations/hw-key/ | yes (soft FIDO2) | fail closed; no IAB action without HW-key |

## ML and AI

none

## Dependencies

- Modules: M14 (bundle status), M20 (inventory), M26 (anomaly flags)
- Platform: P05 (audit), P06 (HSM + FIDO2 hardware key)
- Packages: eel-crypto-rs, eel-outbox-rs

## Feature checklist

1. FR-M15-001: hardware-key (FIDO2) second factor required for all IAB dashboard access.
2. FR-M15-002: real-time election summary: forms received, tokens issued, anomaly count, inventory status.
3. FR-M15-003: reproducibility bundle status: assembly progress, mirror status, download count.
4. FR-M15-004: inventory dashboard: kit count by status (deployed, returned, faulty, lost).
5. FR-M15-005: anomaly and complaint feed: severity-ranked, filterable by module and category.
6. FR-M15-006: escalation workflow: raise, evidence attach, route to IEBC leadership, resolve.
7. FR-M15-007: compliance report generator: statutory format, HSM-signed PDF, anchored to tlog.
8. FR-M15-008: IAB member access management: add/remove members; dual-control for changes.
9. FR-M15-009: audit trail: every IAB action signed and anchored.
10. FR-M15-010: hardware key logistics: provision, distribute, revoke; key ceremony runbook.
11. Dashboard latency <= 5 s from data source to live display.
12. Report generation <= 60 s for standard report types.
13. Hardware key provisioning lead time >= 8 weeks before election day; escalation if short.
14. Dual-control for member access changes (add/remove).

## Acceptance criteria

AC-M15-01: hardware key required; without it, all accredited endpoints return 401 (security test).
AC-M15-02: escalation reaches IEBC leadership alert within 60 s (integration test).
AC-M15-03: compliance report signed and anchored to tlog (integration test).
AC-M15-04: anomaly feed updates within 30 s of M26 event (integration test).
AC-M15-05: dual-control enforced for member management (UAT).
AC-M15-06: 99.9% availability during election cycle (SLO measurement).

## Applicable edge cases

**identity**
EC-IDN-01: hardware key lost; dual-control revocation; emergency access procedure.
EC-IDN-09: dual-control for member changes; re-validate both at commit.
EC-IDN-10: SpiceDB down; fail closed all IAB writes.

**data**
EC-DAT-01: duplicate escalation same key; cached.
EC-DAT-13: resolved escalation re-opened; new escalation_id; chain preserved.

**security**
EC-SEC-01: FIDO2 signature mismatch; reject.
EC-SEC-03: HSM unavailable; report publish blocked; alert.
EC-SEC-16: secrets in IAB feed; telemetry refuses Tier A/B.

**legal**
EC-LEG-02: court injunction; IAB escalation auto-created; chain of custody clear.

**operations**
EC-OPS-05: hardware key ceremony failure; runbook covers remote ceremony fallback.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | FIDO2 auth flow, escalation FSM, report generator, anomaly feed sorter | nextest | M15-UNIT- |
| property | escalation always reaches terminal state; report hash stable | proptest | M15-PROP- |
| contract | M14 bundle status (consumer); M20 inventory (consumer); M26 anomaly (consumer) | Pact | M15-PACT- |
| integration | SoftHSM report sign; soft FIDO2; tlog anchor | nextest + testcontainers | M15-INT- |
| e2e | IAB login -> anomaly review -> escalation -> report publish | ExUnit + Playwright | M15-E2E- |
| security | FIDO2 bypass; ASVS L3; Semgrep; dual-control bypass | Semgrep | M15-SEC- |
| load | concurrent IAB members (50); report generation; feed polling | k6 | M15-LOAD- |

## Runbooks and dashboards

- docs/runbooks/iab-dashboard-degradation.md (create): hardware key logistics; HSM fail; report publish failure; emergency escalation.
- docs/runbooks/iab-key-ceremony.md (create): key provisioning, distribution, and revocation ceremony steps.
- platform/observability/ecc-svc/grafana-iab.json: dashboard latency, escalation queue depth, report generate time, anomaly feed lag.
- platform/observability/ecc-svc/slo-alerts.yaml: 99.9% burn rate; escalation unresolved > 4 h; feed lag > 30 s.

## Open questions

- Hardware key logistics: FIDO2 key provisioning for remote IAB members needs a field distribution plan; current lead-time assumption is 8 weeks before election day.
- Report format: the statutory compliance report format is not yet specified by the Elections Act regulations; IEBC legal team needs to confirm structure before M is implemented.
