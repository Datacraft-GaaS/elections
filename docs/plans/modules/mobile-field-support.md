# M32 mobile-field-support

| field | value |
|---|---|
| M-number | M32 |
| Home service | field-mobile-svc (ADR-0020) |
| Owning work package | WP-4.9 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M32-mobile-field-support.md |

## Purpose

Give every Datacraft field support engineer (~200 on election day) a mobile console for their assigned county or cluster of polling stations: support-ticket queue with SLA timers, remote kit diagnostics, KEBS and warranty-claim initiation, escalation to Datacraft L2/L3 and Mechrevo OEM and the Commission ICT, on-call rotation management, and end-of-shift signed reporting.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Field Support App (Android 10+ / iOS 15+) | Flutter | services/field-mobile-svc/apps/field-support/ |
| API | Rust (axum) | services/field-mobile-svc/crates/field-mobile-api/ |

Distribution: private via Commission MDM (Datacraft device roster). Languages: EN + SW.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| SupportTicket | id, station_code, kit_serial, reporter_role, severity (SEV1/SEV2/SEV3), title, description, status, assigned_engineer_id, sla_target_at, resolved_at, escalation_level | B | T1-consensus |
| DiagnosticAction | id, ticket_id, kit_serial, action_type (FETCH_LOGS/REBOOT/FIRMWARE_ATTEST/SELF_TEST), engineer_id, kit_consent_recorded, signed_at, result | A | T1-audit |
| WarrantyClaim | id, kit_serial, oem_code, fault_description, claim_status, engineer_id, submitted_at | B | T1-consensus |
| ShiftReport | id, engineer_id, cluster_id, shift_start, shift_end, tickets_handled, sla_compliance_pct, escalations, spares_consumed, engineer_signature, submitted_at | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/field/field-support.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/support/tickets | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | n/a | live ticket queue filtered by cluster + severity |
| POST /v1/support/tickets/{id}/take | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | yes | take / assign ticket |
| POST /v1/support/tickets/{id}/escalate | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | yes | escalate to L2/L3/Mechrevo/the Commission ICT |
| GET /v1/support/kits/{serial} | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | n/a | kit health telemetry |
| POST /v1/support/kits/{serial}/action | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | yes | remote diagnostic action (signed, audited, kit-consent required) |
| POST /v1/support/warranty | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | yes | initiate warranty claim |
| GET /v1/support/spares | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | n/a | spares availability in cluster via M20 |
| GET /v1/support/oncall | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | n/a | on-call rotation schedule |
| POST /v1/support/handoff | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | yes | shift boundary handoff; sign-off required |
| POST /v1/support/shift-report | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | yes | submit signed end-of-shift report |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.field.support_ticket_escalated.v1 | emits | eel.field.support.v1 | operational | yes | exactly-once |
| eel.field.diagnostic_action.v1 | emits | eel.field.support.v1 | operational | yes | at-least-once idempotent |
| eel.field.warranty_claim.v1 | emits | eel.field.support.v1 | operational | yes | at-least-once idempotent |
| eel.field.shift_report.v1 | emits | eel.field.support.v1 | operational | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| O02 Support / Ticketing | O02 | integrations/ticketing/ | yes | ticket lifecycle back-end; circuit breaker; queue |
| M20 Inventory Management | internal | integrations/inventory/ | yes | spares lookup and spare-part dispatch |
| O01 MDM | O01 | integrations/mdm/ | yes | private app distribution; remote device management |
| O04 Warranty | O04 | integrations/warranty/ | yes | warranty claim initiation and status tracking |
| M06 BVI Back-End | internal | integrations/bvi/ | yes | kit operational status; incident cross-reference |
| M16 Election Command Centre | internal | integrations/ecc/ | yes | ECC sees engineer location and ticket status; dispatch visibility |

## ML and AI

- None.

## Dependencies

- Modules: M06, M16, M20
- Platform: none
- Packages: kit-sdk-flutter, eel-resilience-rs

## Feature checklist

1. FR-M32-001: staff enrolment via Datacraft engineer ID + biometric template capture + Play Integrity or App Attest device attestation; attestation failure blocks enrolment and emits tamper event
2. FR-M32-002: cluster assignment: engineer bound to specific counties and constituencies for the election; cross-cluster access denied; assignment visible immediately after MDM provisioning
3. FR-M32-003: MFA sign-in: platform biometric (fingerprint or face) + 6-digit PIN + device attestation; all three required; biometric failure falls back to PIN with rate limit of 5 attempts then 30-minute lockout
4. FR-M32-004: live ticket queue filtered by cluster, severity, and age; filtered list refreshes <= 2 s over LTE; severity badge colour-coded and labelled for accessibility
5. FR-M32-005: SEV 1/2/3 SLA countdown timers visible per ticket: SEV 1 response 2 h resolution 1 day, SEV 2 response 4 h resolution 2 days, SEV 3 response 6 h resolution best-effort; timers turn amber at 80% elapsed and red at 100%
6. FR-M32-006: take, assign, and re-assign ticket with dual push notification to old and new assignee; re-assignment recorded in ticket history with reason
7. FR-M32-007: in-app threaded messaging between engineer and ticket reporter (Presiding Officer or county ICT); attachments allowed; messages retained in ticket history
8. FR-M32-008: evidence attachment per ticket: photo, short video (<=60 s), and captured diagnostic log file; per-file size limit enforced; files signed and uploaded to MinIO via field-mobile-api
9. FR-M32-009: kit lookup by serial number (typed) or QR code scan; look up across engineer's cluster; cross-cluster lookup requires explicit reason and is audit-logged
10. FR-M32-010: live kit health telemetry panel per kit: battery percent, connectivity state, GPS fix quality, temperature, error count, last-seen timestamp; data sourced from M06 BVI back-end
11. FR-M32-011: remote diagnostic actions subject to policy engine and explicit kit-side consent: fetch logs, request reboot, request firmware attestation, trigger self-test; policy denies actions outside permitted window
12. FR-M32-012: every remote action signed by engineer biometric at invocation time; action record written to eel.field.support.v1 topic immediately; kit-side confirmation notification sent to station Presiding Officer for privileged actions (reboot, attestation)
13. FR-M32-013: warranty status check per kit serial via O04; spares availability per constituency store via M20 IMS API; both shown on kit-detail screen within 5 s on LTE
14. FR-M32-014: warranty claim filing: OEM automatically identified from kit metadata; claim drafted with fault description, photo evidence, and engineer ID; submitted to O04 and Mechrevo via integrations/warranty/
15. FR-M32-015: warranty claim status tracked in app; push notification when status changes; claim history retained for the election lifecycle
16. FR-M32-016: spare-part shipment and hot-swap kit dispatch coordinated via M20 IMS API; nearest spare-holding constituency store shown on map; dispatch order created from app
17. FR-M32-017: one-tap escalation to Datacraft L2, L3, Mechrevo OEM, and the Commission ICT per pre-configured escalation matrix; escalation level determined by ticket severity and elapsed SLA
18. FR-M32-018: escalation carries full ticket context: description, attachments, all threaded messages, kit telemetry snapshot at time of escalation; no context rebuild needed by recipient
19. FR-M32-019: every escalation logged permanently in eel.field.support_ticket_escalated.v1 event; record is immutable; queryable by the Commission ICT audit trail
20. FR-M32-020: on-call rotation schedule visible per engineer: current on-call, next handoff time, contact details; updated via O02 ticketing back-end
21. FR-M32-021: push notification for new ticket assignment and approaching SLA breach (at 80% and 100% elapsed); notification delivery confirmed via FCM or Apple APNs ack
22. FR-M32-022: shift handoff at shift boundary: outgoing engineer signs off; incoming engineer countersigns; open tickets transferred; handoff record submitted to O02; both signatures required to close shift
23. FR-M32-023: diagnostic runbook library indexed by symptom keyword and kit error code; full-text search returns results in <= 2 s from pre-built on-device index; runbooks versioned and updated via MDM
24. FR-M32-024: contact card for Presiding Officer, Returning Officer, county support desk, war-room hotline, Mechrevo field contact, and the Commission ICT hotline; cached offline
25. FR-M32-025: step-by-step kit reset and recovery guidance embedded offline; covers factory-reset-safe and data-preserving paths; links to relevant runbook section
26. FR-M32-026: nearest spare-holding constituency store identified per cluster via M20; store address, stock count, and contact shown; map pin available offline from cached tile set
27. FR-M32-027: automatic end-of-shift report compiled from local records: tickets handled by severity, SLA compliance percentage, escalations triggered, spares consumed, incidents reported, hours on shift
28. FR-M32-028: shift report signed by engineer biometric; submitted to Datacraft Ops and the Commission ICT via field-mobile-api; queued with UUID7 idempotency key if offline; submission confirmed before session close
29. FR-M32-029: diagnostic runbook library fully offline; no network dependency for any runbook read; runbook version displayed so engineer knows if content may be stale
30. FR-M32-030: ticket queue readable offline from last-sync snapshot; stale-since timestamp displayed; new ticket creation and status updates require connectivity
31. FR-M32-031: remote diagnostic actions queued in Isar + SQLCipher store if kit-gateway unreachable; retried on connectivity restoration with UUID7 idempotency key; no duplicate action executed
32. FR-M32-032: incidents, escalations, and shift reports queued locally with UUID7 idempotency keys; submitted on reconnect; back-end caches response and returns it on re-send
33. FR-M32-033: battery cost <= 5% per hour foreground; background sync bounded to <= 2% per hour; Android Doze and iOS Background App Refresh respected; foreground service used for SLA timer polling only
34. FR-M32-034: mTLS certificate pinning enforced against eel-root CA; captive-portal detection triggers "network blocked" UX; screen-capture protection active on kit-diagnostic screens; no PII in logs, crash reports, or telemetry
35. FR-M32-035: WCAG 2.2 AA on every screen; TalkBack and VoiceOver traversal completes primary ticket-handling flow without dead-end navigation; minimum touch target 48 dp; font scales to 200% without layout break
36. FR-M32-036: offline integrity: Isar WAL journaled before every state mutation including diagnostic action state; OS-kill at any transition resumable on next launch without data loss or duplicate escalation

## Acceptance criteria

1. AC-M32-01: end-to-end ticket taken, worked, escalated, and resolved within SEV 2 SLA on a real Android device; every SLA countdown timer visible and accurate throughout; UAT sign-off with Datacraft engineers
2. AC-M32-02: remote diagnostic action (fetch logs) signed by engineer biometric, audited in eel.field.diagnostic_action.v1 topic, kit-side confirmation sent to Presiding Officer, and audit record queryable in M06; verified in integration test
3. AC-M32-03: SEV 1 escalation routes to Datacraft L2 within 2 h; escalation event carries full ticket context and all attachments; log entry immutable; verified in bench test with O02 stub
4. AC-M32-04: diagnostic runbook library searchable in airplane mode; result returned in <= 2 s on mid-tier Android 10 device; verified in netem offline integration test
5. AC-M32-05: end-of-shift report signed, counter-signed by incoming engineer, submitted to Datacraft Ops and the Commission ICT, and confirmed received; shift handoff requires both signatures; UAT sign-off
6. AC-M32-06: MFA sign-in completes on Android 10+ and iOS 15+ real devices; attestation failure on rooted device blocks sign-in within 2 s and emits tamper event
7. AC-M32-07: mTLS pinning refuses connection to MITM proxy presenting rogue CA cert; zero auth-header bytes observed in captive-portal intercept test
8. AC-M32-08: remote diagnostic action queued while kit-gateway unreachable; retried on reconnect; no duplicate action executed; idempotency key verified by re-sending and confirming single audit record
9. AC-M32-09: 200 engineers filing shift reports simultaneously processed by back-end with p99 <= 2 s and zero dropped reports; k6 load test in staging at election-day peak
10. AC-M32-10: JWT expiry mid-session triggers silent refresh; in-flight ticket-update request completes with 200; next request succeeds; no user re-authentication prompt; Keycloak 5-second TTL test
11. AC-M32-11: WCAG 2.2 AA audit returns zero Priority 1 findings; TalkBack traversal of ticket queue, escalation, and shift-handoff flows completes without workarounds; axe-core scan clean
12. AC-M32-12: no PII (kit serial combined with location, engineer biometric hash) in logs, crash reports, or OTel traces; log-scanner fixture clean in CI
13. AC-M32-13: warranty claim filed with correct OEM code from kit metadata; claim status updated in app on receipt from O04; integration test with O04 stub confirms round-trip
14. AC-M32-14: spares availability at nearest store shown within 5 s of M20 IMS API call on LTE; cached inventory shown with stale-since timestamp when offline
15. AC-M32-15: app build produces byte-identical artefacts across three consecutive CI runs; SBOM (SPDX) generated, cosign-signed, and verifiable; MDM private distribution tested end-to-end

## Applicable edge cases

**mobile**
- EC-MOB-01: OS kills app mid-sync; local Isar WAL intact; queue resumes on next launch without duplicate escalation or report; background-kill harness exercises every sync transition
- EC-MOB-02: storage full on engineer device; app refuses new evidence uploads with graceful error; text-only ticket updates still queued; bounded queue enforced via Isar compaction
- EC-MOB-03: 1 GB RAM device; app heap <= 15 MB; runbook search p99 <= 2 s on 3G; no OOM; verified in device-farm memory gate
- EC-MOB-04: Android Doze or iOS background limits suspend sync; SLA timers continue from local state; queue drains on foreground; no missed escalations
- EC-MOB-05: push token rotated or FCM unavailable; engineer re-registers token silently on next foreground; SLA breach alerts fall back to in-app polling at 60-second interval
- EC-MOB-06: app version below MDM floor; update forced within 48 h; local queue and in-progress tickets survive update; N-1 to N migration verified in integration test
- EC-MOB-07: GPS mock-location on engineer device detected; check-in location flagged spoof-suspect; cluster-assignment validation uses network triangulation as cross-check
- EC-MOB-08: device clock wrong; server-authoritative timestamp used for all SLA timers and audit events; local clock shown as informational only
- EC-MOB-09: biometric sensor broken; PIN fallback rate-limited; no policy downgrade; patrol test with sensor-disabled device confirms lockout enforced

**connectivity**
- EC-CON-01: engineer phone offline; runbook fully accessible; ticket queue read-only from last-sync with stale-since banner; diagnostic actions queued in Isar; escalation falls back to SMS short-code if configured
- EC-CON-02: 3G 500 kbps / 300 ms RTT; ticket queue refresh and escalation submission meet module SLAs; CBOR+zstd negotiated; k6 behind toxiproxy confirms
- EC-CON-05: flapping link during escalation submission; backoff + jitter retry; UUID7 idempotency key prevents duplicate escalation event; toxiproxy toggle test
- EC-CON-06: captive portal intercepts TLS; pinning fails closed; "network blocked" UX shown; no auth tokens emitted; mitmproxy captive-mode test
- EC-CON-08: DNS poisoned; hardcoded fallback endpoints used; circuit opens after 3 failures; engineer notified; DNSChaos test

**identity**
- EC-IDN-02: JWT expires mid-ticket-update; in-flight request completes; silent refresh on next call; no re-authentication prompt; Keycloak 5-second TTL test
- EC-IDN-04: device attestation fails; hard-fail blocks sign-in; support code shown; no partial access granted
- EC-IDN-05: rooted or Frida-attached device; RASP terminates session; tamper event emitted within 2 s; rooted AVD test
- EC-IDN-06: cloned APK with resigned cert; TLS pin mismatch; back-end rejects SVID digest; event emitted to M28
- EC-IDN-07: no SMS OTP for accredited engineers; FIDO2 device-bound credential required; new device denied without MDM re-enrolment
- EC-IDN-10: SpiceDB unavailable; write operations fail closed with 503; read-only queue view continues from cache; PodChaos test
- EC-IDN-11: Keycloak realm key rotated; JWKS refetched on kid miss; next request succeeds without user action

**data**
- EC-DAT-01: duplicate escalation event with same UUID7 idempotency key and body; back-end returns cached 201; no duplicate in eel.field.support.v1 topic
- EC-DAT-02: same UUID7 key with different body; back-end returns 422 idempotency-key-reuse; app surfaces conflict error to engineer
- EC-DAT-09: kit serial and station names with Unicode characters; NFC normalisation before storage; display uses ICU collation; no truncation in runbook or ticket UI

**security**
- EC-SEC-05: all text inputs (ticket description, diagnostic notes) parameterised server-side; no injection path to back-end; Semgrep and ZAP scans clean
- EC-SEC-11: biometric hash comparison uses constant-time compare; no timing oracle on engineer attestation path
- EC-SEC-16: no PII (engineer biometric hash, kit serial combined with GPS) in logs, crash reports, or OTel traces; log-scanner CI gate passes

**scale**
- EC-SCL-04: consumer lag on eel.field.support.v1 topic exceeds 60 s; back-end pages on-call; backpressure applied; app queue bounded; no data loss confirmed

**operations**
- EC-OPS-02: kit unreachable for remote diagnostic action; action queued; escalation triggered automatically after N minutes per SEV SLA matrix; engineer notified
- EC-OPS-06: SEV 1 SLA breach detected; automatic escalation cascade to Datacraft regional lead and the Commission ICT; cascade logged immutably in eel.field.support_ticket_escalated.v1

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | SLA timer, escalation matrix routing, diagnostic action audit, spares lookup | nextest + flutter_test | M32-UNIT- |
| contract | ticketing O02, inventory M20, warranty O04, BVI M06 | Pact | M32-PACT- |
| integration | full ticket cycle; remote diagnostic with kit-consent; warranty claim; shift handoff | integration_test | M32-INT- |
| e2e | engineer check-in, take ticket, run diagnostic, escalate, shift report | Flutter driver | M32-E2E- |
| security | all diagnostic actions signed and audited; no action without kit consent | ZAP + audit-trace | M32-SEC- |
| offline | runbook offline; ticket queue read-only offline | netem | M32-OFFLN- |

Tags: `testWidgets('diagnostic action [FR-M32-011,AC-M32-02]')`

## Runbooks and dashboards

- `docs/runbooks/field-mobile-svc-degradation.md` - extend with field-support offline and SLA-breach paths
- Grafana dashboard: `platform/observability/field-mobile-svc/field-support-dashboard.json`
- SLO alerts: `platform/observability/field-mobile-svc/field-support-slo-alerts.yaml` (SEV1 response <= 2 h)

## Open questions

- O02 Support / Ticketing back-end: scope and provider for ticketing system must be confirmed; MeGuard-derived system or commercial ITSM; integration contract required before M32 integration sprint.
- O04 Warranty back-end: Mechrevo OEM API for warranty claim submission; API spec and authentication must be obtained before integration sprint.
- Kit diagnostic action policy engine: who defines the policy (the Commission ICT or Datacraft SOC)? Policy schema must be agreed before remote-action feature is built.
