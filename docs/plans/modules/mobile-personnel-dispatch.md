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

1. FR-M33-001: worker enrolment via IEBC HR staff ID + biometric template capture (fingerprint or face) on device; enrolment record signed and submitted to O05 HR back-end; duplicate enrolment detected and rejected
2. FR-M33-002: MFA sign-in: platform biometric preferred; 6-digit PIN fallback rate-limited to 5 attempts then 30-minute lockout; session token bound to device attestation; no SMS OTP for supervisors
3. FR-M33-003: DPA consent screen at first launch for location and attendance capture; consent text in EN + SW; required before any GPS use; worker can withdraw consent (triggers ABSENT status and HR notification)
4. FR-M33-004: device attestation via Play Integrity or App Attest; soft-warn on citizen-tier devices (MEETS_BASIC); hard-fail for supervisor roles (MEETS_STRONG); attestation verdict logged per session
5. FR-M33-005: push notification delivered to worker when assigned to a station for an election; notification contains only assignment ID and event type; no PII in FCM/APNs payload
6. FR-M33-006: assignment detail screen: station name, code, GPS coordinates, report time, role, supervisor name and contact, link to reporting documents; cached locally for offline access
7. FR-M33-007: worker can accept or decline assignment with mandatory reason if declined; decline recorded in O05 and triggers substitution flow; accepted state confirmed to supervisor in <= 1 min
8. FR-M33-008: substitution flow managed in O05 HR back-end; app receives real-time update when a new assignment replaces a declined one; old assignment marked REASSIGNED in local Isar store
9. FR-M33-009: GPS check-in on arrival: biometric captured + GPS fix obtained; GPS distance validated against station polygon boundary; check-in accepted only within polygon plus configurable tolerance
10. FR-M33-010: check-in refused if worker is outside polygon threshold; alert issued to supervisor; worker instructed to move to correct location; reason field required if supervisor overrides
11. FR-M33-011: check-in timestamp signed with device key and worker biometric hash; submitted as eel.field.worker_checked_in.v1 event with UUID7 idempotency key
12. FR-M33-012: supervisor receives push notification within 60 s of each worker check-in; supervisor dashboard updates in real time via WebSocket; late check-in (past report time) highlighted
13. FR-M33-013: automatic status pings every 15 minutes while worker is on session; ping contains ON_STATION/BREAK/ABSENT state and reason; missed pings create gap entry in audit log with last-seen timestamp
14. FR-M33-014: explicit status screen: ON_STATION, BREAK (with expected return time), ABSENT (with mandatory reason); status change submitted immediately; status visible to supervisor
15. FR-M33-015: supervisor live-view dashboard shows all team members by station with current status, check-in time, last ping time, and any open incidents; filterable by role and status
16. FR-M33-016: quick-report incident form with categories MEDICAL, SECURITY, KIT_ISSUE, VOTER_DISPUTE, ELECTORAL_VIOLATION, OTHER; free-text description; photo and short-video evidence upload; GPS captured at submission
17. FR-M33-017: escalation chain per incident severity: ROUTINE to supervisor, HIGH to county lead, CRITICAL to war-room; escalation target determined by severity matrix in ECC policy bundle; escalation immutable once set
18. FR-M33-018: end-of-shift check-out with biometric capture + GPS fix; check-out blocked if check-in not confirmed in back-end; check-out timestamp signed with UUID7 idempotency key
19. FR-M33-019: check-out triggers eel.field.payroll_triggered.v1 event delivered exactly-once to IEBC Finance integration; duplicate triggers prevented by UUID7 idempotency key; Finance notified of queue size on burst
20. FR-M33-020: summary card shown immediately after check-out: hours worked, incidents reported, next assignment if any, payroll trigger status (PENDING/SUBMITTED/CONFIRMED); Finance reference number shown on CONFIRMED
21. FR-M33-021: push notifications for: assignment received, reassignment, supervisor broadcast message, incident escalation acknowledgement, payroll trigger confirmed; notification delivery confirmed via FCM/APNs ack
22. FR-M33-022: no PII in push notification payloads; all personal data fetched authenticated from back-end on notification receipt; payload contains only event type and record ID
23. FR-M33-023: assignment cached in Isar + SQLCipher local store; viewable offline with stale-since timestamp; no dependency on back-end connectivity for viewing accepted assignment
24. FR-M33-024: check-in and check-out captured locally before network submission; signed with device key; queued in Isar with UUID7 idempotency key; submitted on reconnect; back-end caches response on re-send
25. FR-M33-025: timestamp integrity: server-authoritative time used for all signed events; local monotonic clock used only for sequencing between server syncs; server-side verification rejects events with skew > 5 min
26. FR-M33-026: EN + SW language support at launch; language preference persists across sessions; date, time, and number formatting use Kenyan locale (ICU); no hard-coded user-facing strings
27. FR-M33-027: WCAG 2.2 AA on every screen including supervisor dashboard and Finance payroll panel; axe-core scan and independent audit both pass; minimum touch target 48 dp
28. FR-M33-028: voice-guided workflow for low-literacy workers on check-in and check-out screens: large icons, numbered steps with audio narration in EN + SW; guided by on-device TTS engine; no network required for audio
29. FR-M33-029: APK size <= 30 MB; app runs on Android 8+ ARM32 and ARM64; single-binary release build; iOS 14+ supported; PWA fallback served at same domain for workers without native app
30. FR-M33-030: PWA fallback for workers without native Android or iOS app; same check-in and check-out flow via browser; GPS and biometric-prompt via browser APIs; offline via service-worker cache
31. FR-M33-031: APK direct-download on the Commission's public website with SHA-256 fingerprint published for sideload path; cosign-signed SBOM (SPDX) published alongside APK
32. FR-M33-032: battery cost <= 3% per hour foreground; GPS sampling rate reduced to once per 5 minutes during status-ping interval under 20% battery; background service bounded to platform limits; Android Doze respected
33. FR-M33-033: mTLS certificate pinning enforced against eel-root CA on all devices; soft-warn "connection unverified" on citizen-tier devices where pinning cannot hard-fail; hard-fail (app closes) for supervisor roles
34. FR-M33-034: SQLCipher AES-256-GCM on all local Isar stores; encryption key stored in hardware-backed keystore on Android 8+ or Apple Secure Enclave on iOS; software keystore only if hardware unavailable with warning logged
35. FR-M33-035: GPS anti-spoof: IMU cross-check applied where sensor available; cell-tower and Wi-Fi triangulation used as cross-check; location flagged spoof-suspect if GPS versus triangulation delta > 1 km; supervisor alerted
36. FR-M33-036: USSD path for status ping from feature phone via I08 gateway; numeric menu <= 5 steps: 1 = ON_STATION, 2 = BREAK, 3 = ABSENT; IVR confirmation message sent to worker; USSD session logs retained
37. FR-M33-037: SMS notification to worker for new assignment on feature phone via I08; SMS contains station name, code, report time, supervisor name; no ID numbers or biometric data in SMS; delivery ack required
38. FR-M33-038: supervisor dashboard shows real-time team check-in rate, absent count, late count, and break count per station; updated via WebSocket; stale indicator shown if WebSocket disconnected > 60 s
39. FR-M33-039: HR dispatcher console (Phoenix LiveView) shows live attendance feed per station; filterable by county, constituency, and role; reassignment action available per worker row; CSV export for Finance
40. FR-M33-040: HR dispatcher can bulk-reassign workers from a high-absence station to another within cluster; bulk-reassignment event logged per worker in eel.field.dispatch.v1; new push notifications sent per affected worker
41. FR-M33-041: worker no-show alert generated after configurable grace period past report time; supervisor receives push notification; HR dispatcher sees alert in console; alert auto-escalates to county lead if unresolved after second threshold
42. FR-M33-042: assignment conflict detection: two workers assigned same station-role pair surfaced as warning in O05 HR back-end at assignment-creation time; M33 app shows warning if conflict exists at check-in
43. FR-M33-043: multi-election assignment support: worker may have accepted assignments in different elections within same session; assignment list shows election name and date; switching election clears GPS cache for prior session
44. FR-M33-044: payroll trigger retry on IEBC Finance integration failure: trigger queued in Isar; retried with exponential backoff; Finance alerted when queue exceeds 100 pending triggers; no duplicate trigger on retry
45. FR-M33-045: worker views payroll trigger status per assignment: PENDING, SUBMITTED, CONFIRMED, FAILED; Finance reference number and processed timestamp shown on CONFIRMED; worker notified via push on status change
46. FR-M33-046: supervisor can broadcast message to all team members at a station via app; message delivered via push + SMS fallback for feature-phone workers; no PII in broadcast payload; message logged in audit trail
47. FR-M33-047: emergency panic alert hidden under long-press gesture on status screen; alert delivered as eel.field.worker_incident.v1 with severity PANIC to supervisor and war-room; GPS captured at trigger; false-alarm acknowledgement required within 2 min
48. FR-M33-048: complete audit trail for all assignment changes, reassignments, check-in overrides, and payroll adjustments; every event signed in eel.field.dispatch.v1 topic; immutable; queryable by HR dispatcher and IEBC ICT
49. FR-M33-049: screen-capture protection (FLAG_SECURE on Android) active on worker personal details and biometric screens; no worker ID or biometric hash in logs, crash reports, or OTel traces; log-scanner CI fixture enforces
50. FR-M33-050: UUID7 idempotency keys on all write operations: check-in, status ping, check-out, incident report, payroll trigger, assignment accept/decline; back-end caches and returns identical response on re-send
51. FR-M33-051: offline integrity: Isar WAL journaled before every state mutation including mid-check-in steps; OS-kill at any transition resumable on next launch without data loss or duplicate payroll trigger; power-cut harness validates every FSM state
52. FR-M33-052: 250,000 concurrent check-ins processed within 90-minute window; back-end Rust axum worker pool auto-scales; Kafka eel.field.dispatch.v1 consumer scales horizontally; p99 check-in response <= 2 s at peak
53. FR-M33-053: check-in flow completes in <= 30 s end-to-end on LTE including biometric capture, GPS fix, signature, and back-end confirmation; benchmark enforced in integration test harness
54. FR-M33-054: status pings delivered at <= 15-minute interval while session active; on connectivity restoration, pings for offline gap submitted with original signed timestamps; gap entry created if no ping received for > 20 min
55. FR-M33-055: worker reassignment received as real-time push update; app prompts worker to accept new station; old assignment marked REASSIGNED in local store with reason; supervisor of new station notified
56. FR-M33-056: IEBC Finance payroll panel (Django) shows paginated list of payroll triggers per election; filterable by county, role, and status; CSV export; FINANCE role only; read-only; no write access from Finance panel
57. FR-M33-057: SpiceDB enforces role hierarchy: WORKER can only submit own check-in and check-out; HR_DISPATCHER can reassign within cluster; FINANCE can only read payroll triggers; SUPERVISOR can override check-in with dual-custody approval; all role checks enforced at API level
58. FR-M33-058: all events in eel.field.dispatch.v1 topic signed with producer SVID; consumer verifies signature before processing; tampered or unsigned events routed to DLQ with tamper alert
59. FR-M33-059: OTA app update: Android Managed Google Play silent update; iOS App Store update; minimum app version floor enforced by MDM policy bundle; local Isar queue and accepted assignments survive update; N-1 to N schema migration verified
60. FR-M33-060: evidence upload for incidents: photo and short video (<= 60 s); signed and uploaded to MinIO via field-mobile-api with checksum; upload queued in Isar if offline; submitted on reconnect
61. FR-M33-061: in-app check-in help for low-literacy workers: large icon grid, numbered steps 1-4 with colour coding, audio narration using on-device TTS in EN + SW; works offline; accessible via single help button on check-in screen
62. FR-M33-062: TalkBack and VoiceOver traversal completes check-in, status-ping, and check-out flows without dead-end navigation; every interactive element has a content description; tested on Android 10+ and iOS 15+
63. FR-M33-063: data minimisation: GPS coordinates retained only for check-in and check-out events; status pings carry network-triangulated precision <= 100 m only; no continuous background GPS tracking at any time
64. FR-M33-064: ODPC-compliant data deletion workflow: worker submits in-app erasure request after election close; GPS coordinates and biometric hash deleted from local store within 30 days; deletion confirmation pushed to worker and HR dispatcher
65. FR-M33-065: check-out blocked if check-in record not confirmed by back-end; local cache of check-in confirmation used; block shown as informational prompt with retry button; never silent failure
66. FR-M33-066: supervisor override for absent check-in requires dual-custody approval: supervisor biometric + county-lead countersign; override recorded permanently in eel.field.dispatch.v1 with reason and both signatures
67. FR-M33-067: ECC (M16) live attendance feed: M33 emits eel.field.worker_checked_in.v1 and eel.field.worker_checked_out.v1 events consumed by M16; ECC console shows per-station check-in rate and absence count in real time
68. FR-M33-068: assignment notification via I08 SMS for feature-phone workers not running the app; SMS contains assignment ID, station name, report time; worker can reply with USSD accept/decline code
69. FR-M33-069: worker can request supervisor help via in-app chat during active session; message delivered via push notification; chat history retained in ticket record in O05; no PII in push payload
70. FR-M33-070: all remote actions and supervisor overrides carry audit context: who, when, which station, what was the prior state, what is the new state; context attached to eel.field.dispatch.v1 event body
71. FR-M33-071: app build reproducible byte-for-byte across three CI runs; SBOM (SPDX) generated and cosign-signed; Android APK and iOS IPA both covered; MDM managed distribution tested end-to-end
72. FR-M33-072: Finance payroll export includes per-worker record: worker ID (pseudonymised), station code, check-in timestamp, check-out timestamp, hours worked, payroll trigger ID, trigger status; no biometric hash in export

## Acceptance criteria

1. AC-M33-01: end-to-end assignment, check-in, status pings, check-out, and payroll trigger flow completed by a real worker on a real Android device; payroll trigger CONFIRMED by Finance stub; UAT sign-off with IEBC HR team
2. AC-M33-02: 250,000 synthetic worker check-ins processed within 90-minute k6 window; p99 check-in response <= 2 s; zero dropped events; back-end auto-scales; payroll triggers queued correctly; load test run in staging
3. AC-M33-03: offline check-in queued, signed locally, submitted on reconnect, confirmed by back-end without duplicate; idempotency verified by re-send; supervisor sees "check-in confirmed" within 60 s of reconnect; airplane-mode integration test
4. AC-M33-04: payroll trigger fires exactly once on check-out; Finance integration receives trigger with correct worker ID pseudonym, station code, and hours worked; trigger CONFIRMED with Finance reference number; integration test with Finance stub
5. AC-M33-05: GPS off-station check-in refused; alert issued to supervisor within 60 s; worker sees distance-from-station indicator; supervisor override requires dual-custody countersign; verified in field trial with real station polygon
6. AC-M33-06: WCAG 2.2 AA independent audit of app, supervisor console, and Finance panel returns zero Priority 1 findings; TalkBack check-in flow and VoiceOver check-in flow complete without dead ends; axe-core clean
7. AC-M33-07: voice-guided check-in workflow completes end-to-end by a worker who cannot read; audio narration plays at each step; no screen-reading required to complete check-in or check-out; verified in accessibility UAT
8. AC-M33-08: GPS anti-spoof detects mock-location delta > 1 km; check-in flagged spoof-suspect; supervisor alerted within 60 s; check-in not silently accepted; cell/Wi-Fi triangulation cross-check applied; mock-location integration test
9. AC-M33-09: USSD status-ping flow completes in <= 5 USSD menu steps on a feature phone; IVR confirmation sent; ping record created in O05; I08 USSD simulator test
10. AC-M33-10: worker no-show alert fires after grace period with no check-in; supervisor push notification delivered; alert escalates to county lead if unresolved; alert cleared when worker checks in; integration test with O05 stub
11. AC-M33-11: mTLS pinning refuses rogue CA cert; zero auth-header bytes in captive-portal intercept; hard-fail closes session for supervisor-role devices; verified with mitmproxy captive-mode harness
12. AC-M33-12: no PII (biometric hash, GPS coordinates, worker personal details) in logs, crash reports, or OTel traces; log-scanner CI fixture passes for app, supervisor console, and Finance panel
13. AC-M33-13: payroll trigger retry delivers trigger after Finance integration recovers from 503; exactly one trigger confirmed; Finance queue-size alert fires when >= 100 pending; retry integration test with chaos Finance stub
14. AC-M33-14: ODPC data-deletion request submitted in-app; GPS and biometric data deleted from local Isar store within 30 days; deletion confirmed to worker and HR dispatcher; verified in ODPC compliance bench test
15. AC-M33-15: app build byte-identical across three CI runs; SBOM generated and cosign-verified; APK SHA-256 fingerprint published on the Commission's public website; MDM silent update tested on Android managed device

## Applicable edge cases

**mobile**
- EC-MOB-01: OS kills app mid-check-in; Isar WAL intact; on next launch check-in flow resumes at biometric step without restarting GPS; no duplicate check-in submitted; power-cut harness exercises every FSM state
- EC-MOB-02: storage full on worker device; app refuses new evidence photo uploads with graceful error; text-only incident report still queued; bounded Isar queue enforced; verified on filled-disk device
- EC-MOB-03: 1 GB RAM Android 8 KES-10k device; app heap <= 15 MB; check-in p99 <= 2 s on 3G; no OOM crash; device-farm memory gate passes
- EC-MOB-04: Android Doze or iOS background limits; background GPS and status-ping service suspended; on foreground, missed pings submitted with original timestamps; no data loss
- EC-MOB-05: push token rotated or FCM unavailable; worker re-registers token silently on next foreground; assignment notifications fall back to SMS via I08; critical escalations fall back to in-app polling
- EC-MOB-06: app version below MDM floor; update forced within 48 h; local Isar queue and accepted assignments survive update; N-1 to N schema migration verified in integration test
- EC-MOB-07: GPS mock-location detected; cell/Wi-Fi delta > 1 km; check-in flagged spoof-suspect; supervisor alerted within 60 s; check-in not silently accepted; mock-location integration test
- EC-MOB-08: device clock wrong; server-authoritative timestamp used for all signed events; server rejects events with skew > 5 min; worker sees "clock sync required" prompt
- EC-MOB-09: biometric sensor broken; PIN fallback activated and rate-limited; no policy downgrade; supervisor notified; patrol test with sensor-disabled device

**connectivity**
- EC-CON-01: worker phone offline >= 48 h; assignment viewable offline; check-in queued in Isar with signed timestamp; submitted on reconnect with UUID7 idempotency key; supervisor sees "check-in pending" until confirmation
- EC-CON-02: 3G 500 kbps / 300 ms RTT; check-in submission and status ping meet SLAs; CBOR+zstd negotiated; k6 behind toxiproxy confirms p99 <= 2 s
- EC-CON-03: 2G only; app shows "limited connectivity" banner; USSD path offered for status ping; assignment viewable offline; no data over 2G without user confirmation
- EC-CON-05: flapping link during check-in submission; backoff + jitter retry; UUID7 idempotency key prevents duplicate event; retry budget consumed before queue overflow; toxiproxy toggle test
- EC-CON-06: captive portal intercepts TLS; pinning fails closed for supervisor-role devices; "network blocked" UX shown; no auth tokens emitted; mitmproxy captive-mode test
- EC-CON-08: DNS poisoned; hardcoded fallback endpoints used; circuit opens; worker notified; DNSChaos test confirms no token leak

**identity**
- EC-IDN-02: JWT expires mid-check-in submission; in-flight request completes; silent refresh on next call; no re-authentication prompt; Keycloak 5-second TTL test
- EC-IDN-04: device attestation fails (custom ROM, MEETS_NONE); hard-fail for supervisor roles blocks sign-in; soft-warn for citizen-tier workers; support code shown; no partial access
- EC-IDN-05: rooted or Frida-attached device; RASP terminates session; SQLCipher keys wiped from memory; tamper event emitted to M06 within 2 s; rooted AVD matrix test
- EC-IDN-06: cloned APK with resigned cert; TLS pin mismatch; back-end rejects SVID digest; event emitted to M28 AI monitoring stack
- EC-IDN-07: no SMS OTP for supervisor-role workers; FIDO2 device-bound credential required; new device denied without MDM re-enrolment; casual-worker OTP is device-bound via Play Integrity binding
- EC-IDN-10: SpiceDB unavailable; write operations fail closed with 503 problem+json; read-only assignment view continues from local cache; PodChaos test
- EC-IDN-11: Keycloak realm key rotated; JWKS refetched on kid miss; next request succeeds without user action; rotation-in-testcontainer test

**data**
- EC-DAT-01: duplicate check-in with same UUID7 key and body; back-end returns cached 201; no duplicate event in eel.field.dispatch.v1 topic; no second payroll trigger on check-out re-send
- EC-DAT-02: same UUID7 key with different check-in body (tampered GPS); back-end returns 422 idempotency-key-reuse; app surfaces conflict error; incident logged
- EC-DAT-09: worker names with Unicode characters; NFC normalisation before signing and storage; ICU collation in supervisor dashboard; no truncation; hypothesis unicode corpus

**security**
- EC-SEC-05: all free-text inputs (incident description, decline reason) parameterised server-side; no injection path to Postgres or event store; Semgrep and ZAP scans clean
- EC-SEC-11: biometric hash comparison uses constant-time compare; no timing oracle on check-in attestation path; dudect-style timing test in CI
- EC-SEC-16: no PII (worker biometric hash, GPS coordinates, personal details) in structured logs, crash reports, or OTel traces; log-scanner fixture passes for app, supervisor console, and Finance panel

**scale**
- EC-SCL-04: consumer lag on eel.field.dispatch.v1 topic exceeds 60 s; back-end pages on-call; backpressure applied to check-in ingest; app queue bounded to prevent OOM; no data loss confirmed by slow-consumer chaos test
- EC-SCL-07: 250,000 check-ins at 1x scale with p99 <= 2 s; <= 10% p99 regression at 5x (250k concurrent x 5 waves) in simulator pre-production run; Kafka horizontal scale verified

**human**
- EC-HUM-06: WCAG 2.2 AA on every screen; TalkBack check-in and check-out flows complete without workarounds; axe-core scan and independent manual audit both pass
- EC-HUM-08: EN + SW UI; SMS and USSD fallback for feature-phone workers in SW; all strings tested in 150% pseudo-locale; ICU plural forms for "1 worker" vs "N workers"
- EC-HUM-10: low-literacy USSD path for status ping in <= 5 numeric steps; IVR alternative; USSD simulator walk confirms end-to-end

**legal**
- EC-LEG-02: DPA consent for location capture obtained at first launch in EN + SW; consent record retained per DPA 2019 Art.35; worker can withdraw via in-app action; withdrawal triggers ABSENT status and HR notification; ODPC audit trail generated

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
