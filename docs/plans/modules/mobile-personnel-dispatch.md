# M33 mobile-personnel-dispatch

| field | value |
|---|---|
| M-number | M33 |
| Home service | field-mobile-svc (ADR-0020) |
| Owning work package | WP-4.9 (docs/plans/05-wave-4-operations.md) |
| Size | XL |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M33-mobile-personnel-dispatch.md |

## Purpose

Dispatch, attend, and track every polling-day worker (~250,000 POs, DPOs, Clerks, RO staff, security, cleaners, drivers) across ~46,000 polling stations: deliver assignment, GPS-verify check-in, on-site status pings every 15 minutes, incident reporting, GPS-verified check-out with payroll trigger; successor to Datacraft's MeGuard pattern proven for large field-workforce management.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Personnel Dispatch App (Android 8+ / iOS 14+ / PWA fallback) | Flutter | services/field-mobile-svc/apps/personnel-dispatch/ |
| IEBC HR Dispatcher Console | Phoenix LiveView | services/field-mobile-svc/ex/dispatch_web/ |
| IEBC Finance Payroll Panel | Django | services/field-mobile-svc/django/payroll_admin/ |
| API | Rust (axum) | services/field-mobile-svc/crates/field-mobile-api/ |

Distribution: private via IEBC MDM + signed APK for casual-worker devices. Languages: EN + SW; voice-guided workflow for low-literacy users.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| WorkerAssignment | id, worker_id, station_code, election_id, role, report_time, supervisor_id, status (PENDING/ACCEPTED/DECLINED/REASSIGNED), accepted_at | B | T1-consensus |
| WorkerCheckIn | id, worker_id, station_code, election_id, biometric_hash, gps_lat, gps_lon, gps_accuracy_m, check_in_at, signature | A | T1-audit |
| WorkerStatusPing | id, worker_id, station_code, status (ON_STATION/BREAK/ABSENT), reason, pinged_at | C | T2 |
| WorkerCheckOut | id, worker_id, station_code, election_id, biometric_hash, gps_lat, gps_lon, check_out_at, hours_worked, payroll_trigger_id, signature | A | T1-audit |
| WorkerIncidentReport | id, worker_id, station_code, incident_type, description, evidence_blob_ids, severity, escalation_targets, submitted_at, signature | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/field/personnel-dispatch.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/dispatch/assignments | 2 | JWT+SpiceDB (WORKER) | n/a | worker's assignment list |
| POST /v1/dispatch/assignments/{id}/accept | 2 | JWT+SpiceDB (WORKER) | yes | accept assignment |
| POST /v1/dispatch/assignments/{id}/decline | 2 | JWT+SpiceDB (WORKER) | yes | decline with reason; triggers HR substitution flow |
| POST /v1/dispatch/check-in | 2 | JWT+SpiceDB (WORKER) | yes | biometric + GPS station check-in |
| POST /v1/dispatch/status | 2 | JWT+SpiceDB (WORKER) | yes | on-site status ping (ON_STATION/BREAK/ABSENT) |
| POST /v1/dispatch/check-out | 2 | JWT+SpiceDB (WORKER) | yes | biometric + GPS check-out; triggers payroll |
| POST /v1/dispatch/incidents | 2 | JWT+SpiceDB (WORKER) | yes | quick-report incident |
| GET /v1/dispatch/hr/stations | 2 | JWT+SpiceDB (HR_DISPATCHER) | n/a | live attendance feed per station |
| POST /v1/dispatch/hr/reassign | 2 | JWT+SpiceDB (HR_DISPATCHER) | yes | reassign worker to different station |
| GET /v1/dispatch/finance/payroll-triggers | 2 | JWT+SpiceDB (FINANCE) | n/a | list payroll triggers for processing |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.field.worker_checked_in.v1 | emits | eel.field.dispatch.v1 | operational | yes | at-least-once idempotent |
| eel.field.worker_checked_out.v1 | emits | eel.field.dispatch.v1 | operational | yes | at-least-once idempotent |
| eel.field.worker_incident.v1 | emits | eel.field.dispatch.v1 | operational | yes | at-least-once idempotent |
| eel.field.payroll_triggered.v1 | emits | eel.field.dispatch.v1 | operational | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| O05 Personnel Dispatch Back-End | O05 | integrations/personnel-backend/ | yes | system of record; circuit breaker; offline queue |
| M06 BVI Back-End | internal | integrations/bvi/ | yes | polling-station status cross-reference at check-in |
| M16 Election Command Centre | internal | integrations/ecc/ | yes | ECC sees real-time attendance feed; dispatches reassignments |
| M20 Inventory Management | internal | integrations/inventory/ | yes | kit validation at check-in; assigned kit in system |
| IEBC Finance | internal | integrations/finance/ | yes | payroll trigger on check-out; queue on failure; retry; alert Finance |
| I08 SMS / USSD gateway | I08 | integrations/sms/ | yes | OTP fallback; assignment notification to feature-phone workers |

## ML and AI

- None.

## Dependencies

- Modules: M06, M16, M20
- Platform: none
- Packages: kit-sdk-flutter, eel-resilience-rs

## Feature checklist

1. FR-M33-001: enrolment via IEBC HR staff ID + biometric enrolment on device
2. FR-M33-002: MFA sign-in: biometric preferred; PIN fallback
3. FR-M33-003: DPA consent screen for location and attendance capture; required before any GPS use
4. FR-M33-004: device attestation (soft-fail warn on citizen-tier devices; hard-fail for supervisor roles)
5. FR-M33-005: push notification when worker assigned to a station for an election
6. FR-M33-006: assignment detail: station name, code, GPS, report time, role, supervisor contact, reporting docs
7. FR-M33-007: assignment acceptance or decline with reason if declined
8. FR-M33-008: substitution flow: RO can reassign via HR back-office; app updates in real time
9. FR-M33-009: check-in on arrival with biometric + GPS; GPS distance validated against station polygon
10. FR-M33-010: check-in fails if worker > threshold distance from station; alert issued
11. FR-M33-011: check-in timestamp signed and submitted
12. FR-M33-012: supervisor notified on worker check-in
13. FR-M33-013: periodic status pings every 15 min confirm on-station presence
14. FR-M33-014: explicit status screen: On-station, Break, Absent (with reason)
15. FR-M33-015: supervisor live view of team status
16. FR-M33-016: quick-report incident form: medical, security, kit-issue, dispute, other; description; evidence upload
17. FR-M33-017: escalation to supervisor, county lead, war-room per severity
18. FR-M33-018: end-of-shift check-out with biometric + GPS
19. FR-M33-019: check-out triggers payroll milestone via IEBC Finance integration
20. FR-M33-020: summary card on check-out: hours worked, incidents reported, next assignment (if any)
21. FR-M33-021: push notifications: assignment received, reassignment, supervisor message, incident escalation, payroll processed
22. FR-M33-022: no PII in push payloads; content fetched authenticated on receipt
23. FR-M33-023: assignment cached locally; viewable offline
24. FR-M33-024: check-in and check-out captured locally; signed; queued; submitted on reconnect with idempotency keys
25. FR-M33-025: timestamp integrity: signed local time source; server-side verification on reconnect
26. FR-M33-026: EN + SW language support
27. FR-M33-027: WCAG 2.2 AA
28. FR-M33-028: voice-guided workflow for low-literacy users
29. Check-in flow <= 30 s; battery cost <= 3% per hour foreground; 250,000 concurrent check-ins over 90-minute window; runs on Android 8 mid-tier; certificate pinning + attestation (soft) mandatory; GPS anti-spoof with IMU cross-check where available

## Acceptance criteria

1. AC-M33-01: end-to-end assign, check-in, check-out flow; UAT
2. AC-M33-02: 250,000 concurrent check-ins over 90 min; load test on back-end
3. AC-M33-03: offline check-in submits correctly on reconnect; airplane-mode + reconnect test
4. AC-M33-04: payroll trigger fires on check-out; integration test
5. AC-M33-05: GPS off-station check-in refused; field trial
6. AC-M33-06: WCAG 2.2 audit pass; independent audit

## Applicable edge cases

**connectivity**
- EC-CON-01: worker phone offline; assignment cached; check-in queued; submitted on reconnect with idempotency key; supervisor sees "check-in pending"

**identity**
- EC-IDN-09: duplicate check-in prevented by idempotency; alert on multiple devices for same worker

**operations**
- EC-OPS-05: worker off-station; check-in refused; supervisor alerted; worker instructed to move
- EC-OPS-06: GPS unavailable; fallback to network-triangulated location; supervisor override required; reason logged

**legal**
- EC-LEG-02: DPA consent for location capture; consent screen at first use; per-session not stored beyond shift; worker can withdraw

**mobile**
- EC-MOB-01: Android 8 low-battery; check-in flow <= 30 s; reduced polling interval; assignment cached
- EC-MOB-02: 250,000 workers on a mix of Android 8 to 12 devices; APK <= 30 MB; no minimum RAM assumption
- EC-MOB-07: 250,000 concurrent check-ins; back-end worker pool auto-scales; queue absorbs burst

**human**
- EC-HUM-06: WCAG 2.2 AA; voice-guided workflow for low-literacy workers
- EC-HUM-08: EN + SW; SMS notification fallback for workers on feature phones

**scale**
- EC-SCL-01: 250,000 check-ins in 90 min; payroll triggers all queued; Finance notified of queue size

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | GPS distance check, idempotency key, payroll trigger, status-ping interval | nextest + flutter_test | M33-UNIT- |
| property | idempotent check-in, payroll trigger exactly-once delivery | proptest | M33-PROP- |
| contract | O05 personnel back-end, M16 ECC feed, Finance payroll | Pact | M33-PACT- |
| integration | full assign, check-in, ping, check-out, payroll trigger | integration_test | M33-INT- |
| e2e | 1,000 synthetic workers; concurrent check-in; check-out; payroll | Flutter driver | M33-E2E- |
| security | GPS spoof detection; off-station check-in refused; DPA consent gate | ZAP + netem | M33-SEC- |
| a11y | WCAG 2.2 AA; voice-guided workflow | axe-core + manual | M33-A11Y- |
| offline | check-in queued; reconnect integrity | netem | M33-OFFLN- |
| load | 250,000 concurrent check-ins over 90 min | k6 | M33-LOAD- |

Tags: `testWidgets('check-in [FR-M33-009,AC-M33-02,EC-MOB-07]')`

## Runbooks and dashboards

- `docs/runbooks/field-mobile-svc-degradation.md` - extend with personnel-dispatch offline queue and payroll failure paths
- `docs/runbooks/payroll-trigger-failure.md` - new runbook; Finance notification; queue drain; retry SLA
- Grafana dashboard: `platform/observability/field-mobile-svc/personnel-dispatch-dashboard.json`
- SLO alerts: `platform/observability/field-mobile-svc/personnel-dispatch-slo-alerts.yaml` (check-in <= 30 s, payroll trigger exactly-once)

## Open questions

- O05 Personnel Dispatch Back-End: scope as MeGuard-derived system or new service; API contract and data-model agreement required before integration sprint.
- IEBC Finance integration: payroll API schema and authentication; Finance system must be confirmed and API spec delivered before M33 integration sprint; timeline risk.
- Assignment collision resolution: two workers assigned same station-role; HR back-office resolves; conflict-detection must be in O05, not M33; confirm responsibility boundary.
- Feature-phone workers: SMS-only notification path for workers without smartphones; I08 gateway confirmed as sufficient; APK sideload for Android confirmed as fallback.
