# M16 election-command-centre

| field | value |
|---|---|
| M-number | M16 |
| Home service | ecc-svc (ADR-0020) |
| Owning work package | WP-4.1 (docs/plans/05-wave-4-operations.md) |
| Size | XL |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M16-election-command-centre.md |

## Purpose

The National Intelligence and Situational Awareness (NIS) loop for IEBC leadership and operations teams: aggregate telemetry from all kit streams, results, anomalies, grievances, speech flags, queue data, and inventory into a unified command view, enabling real-time dispatch decisions, s.44A authorisations, and coordinated incident response.

## Surfaces and languages

| surface | technology | path under services/ecc-svc/ |
|---|---|---|
| ECC multi-role dashboard | Elixir (Phoenix LiveView) | ex/ecc-live/ |
| Mobile ECC app (field command) | Flutter on packages/kit-sdk-flutter | apps/ecc-mobile/ |
| ECC API | Rust (axum + utoipa) | crates/ecc-api/ |
| Alert + dispatch logic | Rust | crates/ecc-domain/ |
| Incident report generator | Rust + Typst | crates/ecc-reports/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| KitTelemetrySnapshot | snap_id, kit_serial, station_code, battery, connectivity, gps_fix, temp, last_seen, error_codes | C | T2 |
| ECCAlert | alert_id, source_module, severity, category, description, status, assigned_to, resolved_at | C | T2 |
| S44AEvent | event_id, station_code, authorised_by, authorised_at, reason, dual_officer, sig | C | T1-audit |
| Dispatch | dispatch_id, alert_id, personnel_ids, instruction, status, acked_at | C | T2 |
| IncidentReport | report_id, election_id, category, content_hash, sig, published_at | D | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/ecc/command.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /ecc/v1/summary | 3 accredited | JWT+SpiceDB | n/a | Live national summary: kits, alerts, results progress |
| GET /ecc/v1/kits | 3 accredited | JWT+SpiceDB | n/a | Kit telemetry heatmap |
| GET /ecc/v1/alerts | 3 accredited | JWT+SpiceDB | n/a | Alert feed, filterable by severity/module/area |
| POST /ecc/v1/alerts/{id}/dispatch | 3 accredited | JWT+SpiceDB | yes | Dispatch personnel to alert |
| PATCH /ecc/v1/alerts/{id} | 3 accredited | JWT+SpiceDB | yes | Update alert status |
| POST /ecc/v1/s44a | 3 accredited | JWT+SpiceDB | yes | Record s.44A authorisation (dual-officer) |
| GET /ecc/v1/anomalies | 3 accredited | JWT+SpiceDB | n/a | Anomaly/fraud flags from M17/M26 |
| GET /ecc/v1/queue | 3 accredited | JWT+SpiceDB | n/a | Queue status from M19 |
| POST /ecc/v1/reports | 3 accredited | JWT+SpiceDB | yes | Publish incident report |
| GET /ecc/v1/inventory | 3 accredited | JWT+SpiceDB | n/a | Inventory status from M20 |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.ecc.alert_created.v1 | emits | eel.ecc.alert.v1 | operational | yes | at-least-once idempotent |
| eel.ecc.alert_resolved.v1 | emits | eel.ecc.alert.v1 | operational | yes | at-least-once idempotent |
| eel.ecc.s44a_authorised.v1 | emits | eel.ecc.s44a.v1 | operational | yes (dual) | exactly-once |
| eel.ecc.dispatch_created.v1 | emits | eel.ecc.dispatch.v1 | operational | yes | at-least-once idempotent |
| eel.bvi.session_accepted.v1 | consumes | eel.bvi.session.v1 | operational | yes | at-least-once idempotent |
| eel.bvi.distress_received.v1 | consumes | eel.bvi.security.v1 | operational | yes | exactly-once |
| eel.rts.form_ingested.v1 | consumes | eel.rts.form.v1 | event-source | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| BVI back-end M06 | internal | integrations/bvi-backend/ | yes | telemetry feed; circuit breaker |
| RTS back-end M10 | internal | integrations/rts-backend/ | yes | form ingest status feed |
| Verifier app M13 | internal | integrations/verifier/ | yes | grievance + anomaly relay |
| Speech integrity M17 | internal | integrations/speech/ | yes | content flags feed |
| Queue transparency M19 | internal | integrations/queue/ | yes | disenfranchisement flags |
| Inventory M20 | internal | integrations/inventory/ | yes | kit/seal status |
| Anomaly/fraud M26 | internal | integrations/anomaly/ | yes | fraud alert feed |
| Personnel dispatch M33 | internal | integrations/dispatch/ | yes | personnel assignment |
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed on s44A + report sign |

## ML and AI

No ML runs inside this module. It consumes flags from M17 (speech integrity) and M26 (anomaly/fraud) and presents them in the alert feed.

## Dependencies

- Modules: M08, M10, M13, M17, M18, M19, M20, M26, M33
- Platform: P05 (audit), P06 (HSM)
- Packages: eel-crypto-rs, eel-outbox-rs, kit-sdk-flutter

## Feature checklist

1. FR-M16-001: national live summary: kits online/offline, forms received, tokens issued, alerts by severity.
2. FR-M16-002: kit telemetry heatmap: constituency/ward/station; filter by battery/connectivity/GPS/error.
3. FR-M16-003: alert feed: severity-ranked; categories include anomaly, grievance, distress, queue, speech, inventory.
4. FR-M16-004: alert dispatch: assign personnel (M33) to alert; track acknowledgement.
5. FR-M16-005: s.44A authorisation: dual-officer; HSM-signed; reasons logged; propagated to M08/M06.
6. FR-M16-006: distress signal from kit (M06) escalates to alert within 5 s.
7. FR-M16-007: speech integrity flags from M17 displayed with severity and evidence link.
8. FR-M16-008: queue transparency flags from M19 displayed with disenfranchisement risk rating.
9. FR-M16-009: inventory alerts from M20 (missing kit, seal broken, reserves low).
10. FR-M16-010: incident report generator: structured report, HSM-signed, anchored to tlog.
11. FR-M16-011: role-based views: national command, regional, county, field command; no cross-scope visibility.
12. FR-M16-012: NIS loop SLA: all source feeds aggregated and alerts created within 30 s of upstream event.
13. FR-M16-013: mobile ECC app: field command officer receives alerts and dispatches offline-resilient.
14. FR-M16-014: audit trail: every command action signed and anchored.
15. RBAC complexity: role matrix documented and enforced by SpiceDB; >= 8 distinct roles.
16. NIS loop latency <= 30 s from upstream event to alert.
17. Distress alert <= 5 s from M06 distress event to ECC alert.
18. s.44A dual-officer flow requires physically separate auth challenges.
19. 250k dispatch records per election day sustainable.

## Acceptance criteria

AC-M16-01: NIS loop latency <= 30 s from upstream event to alert (integration test).
AC-M16-02: distress alert appears in ECC within 5 s (chaos exercise).
AC-M16-03: s.44A requires dual-officer HSM auth; single officer cannot complete (UAT).
AC-M16-04: RBAC: county user cannot see national alerts (security test).
AC-M16-05: incident report signed and anchored (integration test).
AC-M16-06: mobile app receives alert offline; acks on reconnect (chaos test).
AC-M16-07: 99.9% availability during election cycle (SLO measurement).

## Applicable edge cases

**connectivity**
EC-CON-04: source feed disconnect; alert: feed-lost; stale labelled.
EC-CON-05: flapping source feeds; debounce; no duplicate alerts.

**time**
EC-TIM-03: HLC on all aggregated events; no regression.
EC-TIM-07: extended tallying; s.44A extension propagated.

**identity**
EC-IDN-01: officer cert revoked; sessions invalidated; ECC locked to admin re-provision.
EC-IDN-09: s.44A dual-officer re-validate at commit.
EC-IDN-10: SpiceDB down; fail closed all write operations.

**data**
EC-DAT-01: duplicate alert same key; idempotent.
EC-DAT-14: alert storm (>1000/min); rate-limit DLQ; aggregated digest to ECC.

**security**
EC-SEC-01: HSM mismatch on s44A; blocked.
EC-SEC-03: HSM unavailable; s44A and report blocked; alert.
EC-SEC-16: secrets in alert descriptions refused.

**human**
EC-HUM-08: 11 languages in alert descriptions.

**scale**
EC-SCL-03: alert hot partition; spread by station_code.
EC-SCL-07: 250k dispatches per day; p95 <= 2 s.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | alert aggregation, dispatch FSM, s44A dual-officer flow, NIS loop debounce, RBAC policy | nextest + Flutter test | M16-UNIT- |
| property | alert always reaches terminal state; NIS latency <= 30 s under load; distress <= 5 s | proptest | M16-PROP- |
| contract | M06 telemetry (consumer); M17 speech flags (consumer); M26 anomaly (consumer); M33 dispatch (provider) | Pact | M16-PACT- |
| integration | SoftHSM s44A; all source doubles; distress routing | nextest + testcontainers | M16-INT- |
| e2e | distress -> ECC alert -> dispatch -> ack; s44A dual sign | ExUnit + patrol | M16-E2E- |
| fuzz | alert payload; dispatch instruction; s44A reason | cargo-fuzz | M16-FUZZ- |
| load | 250k dispatches; alert storm; 8-role RBAC | k6 | M16-LOAD- |
| chaos | source feed kill; HSM kill; SpiceDB kill | Chaos Mesh | M16-CHAOS- |
| security | RBAC bypass; s44A single-officer; ASVS L3; Semgrep | Semgrep | M16-SEC- |

## Runbooks and dashboards

- docs/runbooks/ecc-svc-degradation.md (create): source feed disconnect; HSM fail; SpiceDB outage; alert storm mitigation.
- docs/runbooks/ecc-s44a-procedure.md (create): dual-officer s.44A authorisation steps; remote ceremony fallback.
- platform/observability/ecc-svc/grafana-ecc.json: NIS loop latency, alert throughput, dispatch ack rate, feed lag by source.
- platform/observability/ecc-svc/slo-alerts.yaml: 99.9% burn rate; NIS loop > 30 s; distress > 5 s; alert unresolved > 2 h.

## Open questions

- RBAC role matrix: the 8+ roles (national, regional, county, field, legal, IAB, press, observer) need formal sign-off from IEBC HR and legal before implementation.
- NIS loop aggregation: consolidating 14+ source feeds into a single coherent alert model needs an architecture decision on priority weighting; current design uses static severity weights.
