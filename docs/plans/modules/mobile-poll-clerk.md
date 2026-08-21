# M31 mobile-poll-clerk

| field | value |
|---|---|
| M-number | M31 |
| Home service | field-mobile-svc (ADR-0020) |
| Owning work package | WP-4.8 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M31-mobile-poll-clerk.md |

## Purpose

Give the Deputy Presiding Officer and polling clerks (~92,000 on polling day) a companion tool that offloads work from the KIEMS device: searchable reference materials, structured incident logging, voter-lookup fallback when the kit is single-user busy, queue management, mirrored kit dashboards, paper-backup workflow guidance, and end-of-day report generation; the KIEMS kit remains authoritative for voter identification, and this app makes the team around it faster and less error-prone.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Poll Clerk Companion App (Android 10+ / iOS 15+) | Flutter | services/field-mobile-svc/apps/poll-clerk/ |
| Kiosk Operator Mode (register inspection period) | Flutter | services/field-mobile-svc/apps/poll-clerk/ |
| API | Rust (axum) | services/field-mobile-svc/crates/field-mobile-api/ |

Distribution: private via IEBC MDM. Languages: EN + SW.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| ClerkCheckIn | id, clerk_id, station_code, election_id, role (DPO/CLERK/KIOSK), biometric_hash, device_attestation_id, check_in_at | B | T1-consensus |
| VoterLookupAudit | id, clerk_id, station_code, lookup_reason, id_hash (no voter PII), dpo_attestation, looked_up_at | A | T1-audit |
| ClerkIncidentReport | id, clerk_id, station_code, incident_type, description, evidence_blob_ids, escalation_level, submitted_at, signature | B | T1-consensus |
| EndOfDayReport | id, station_code, clerk_id, opening_time, closing_time, turnout_count, incidents_count, spoilt_paper_log, agent_presence_log, dpo_signature, po_signature, submitted_at | A | T1-audit |

## APIs

OpenAPI file: `packages/openapi-contracts/field/poll-clerk.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/clerk/check-in | 2 | JWT+SpiceDB (DPO/CLERK) | yes | biometric + device attestation station check-in |
| GET /v1/clerk/station/{code}/dashboard | 2 | JWT+SpiceDB (DPO/CLERK) | n/a | read-only kit dashboard mirror from BVI back-end |
| POST /v1/clerk/voter-lookup | 2 | JWT+SpiceDB (DPO) | yes | voter register lookup fallback; rate-limited; logged |
| POST /v1/clerk/incidents | 2 | JWT+SpiceDB (DPO/CLERK) | yes | submit structured incident report |
| POST /v1/clerk/queue | 2 | JWT+SpiceDB (DPO/CLERK) | yes | manage voter queue (add, next, priority) |
| GET /v1/clerk/queue/{station_code} | 2 | JWT+SpiceDB (DPO/CLERK) | n/a | queue status |
| POST /v1/clerk/end-of-day | 2 | JWT+SpiceDB (DPO) | yes | submit signed end-of-day report |
| GET /v1/clerk/kiosk/register | 2 | JWT+SpiceDB (KIOSK_OPERATOR) | n/a | BVV register lookup in kiosk mode; staff-assisted OTP |
| POST /v1/clerk/kiosk/objection | 2 | JWT+SpiceDB (KIOSK_OPERATOR) | yes | file objection on behalf of citizen with consent |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.field.clerk_checked_in.v1 | emits | eel.field.clerk.v1 | operational | yes | at-least-once idempotent |
| eel.field.clerk_incident.v1 | emits | eel.field.clerk.v1 | operational | yes | at-least-once idempotent |
| eel.field.end_of_day_report.v1 | emits | eel.field.clerk.v1 | operational | yes | exactly-once |
| eel.field.voter_lookup_audit.v1 | emits | eel.field.clerk.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M06 BVI Back-End | internal | integrations/bvi/ | yes | dashboard mirror; incidents submitted; circuit breaker |
| M02 BVR Back-End | internal | integrations/bvr/ | yes | voter register lookup fallback; offline unavailable |
| M07 BVV | internal | integrations/bvv/ | yes | kiosk-mode register lookup and objection filing |
| M20 Inventory Management | internal | integrations/inventory/ | yes | kit self-check on app launch; assigned kit in inventory |

## ML and AI

- None.

## Dependencies

- Modules: M02, M06, M07, M20
- Platform: none
- Packages: kit-sdk-flutter, eel-resilience-rs

## Feature checklist

1. FR-M31-001: staff enrolment via IEBC staff ID + biometric template capture + Play Integrity or App Attest device attestation; attestation failure blocks enrolment and emits a tamper event
2. FR-M31-002: assignment scope bound to a specific station code and election ID; cross-station access denied; dual-role (DPO + KIOSK_OPERATOR) allowed only where HR MDM policy grants it
3. FR-M31-003: MFA sign-in: platform biometric (fingerprint or face) + 6-digit PIN + device attestation; all three required; biometric failure falls back to PIN with rate limit of 5 attempts then 30-minute lockout
4. FR-M31-004: polling manual (EN + SW) embedded offline; full-text search returns first result in <= 1 s from pre-built on-device index; manual versioned and updated via signed MDM push only
5. FR-M31-005: statutory forms library (Form 34A/B/C templates) embedded offline for paper-backup workflow; forms exportable as PDF; version pinned to election bundle
6. FR-M31-006: Elections Act key clauses (s.55 emergency close, s.6A cut-off, s.44A assisted voting) surfaced in searchable reference panel; citations link to manual page
7. FR-M31-007: contact card with Presiding Officer, Returning Officer, County Returning Officer, county support desk, war-room hotline, Kenya Red Cross emergency; cached offline; updated via MDM
8. FR-M31-008: live dashboard mirroring BVI back-end (M06): election name, opening and closing times, turnout count, voters authenticated, kit battery percent, connectivity state, GPS fix quality; refreshed at <= 30 s interval
9. FR-M31-009: dashboard strictly read-only from M06; no writes to authentication pipeline; dashboard read errors surfaced as informational banners, not blocking alerts; WebSocket or long-poll transport
10. FR-M31-010: voter register lookup fallback by national ID when KIEMS kit is temporarily unavailable; requires DPO biometric attestation and written justification captured at each lookup
11. FR-M31-011: lookup rate-limited per station per hour (limit set by ECC policy bundle); every lookup recorded permanently with DPO biometric attestation hash, timestamp, reason, and station code; no lookup without attestation
12. FR-M31-012: app provides information lookup only; voter authentication is never performed from this app; KIEMS kit remains the sole authoritative identification device; UI copy enforces this distinction
13. FR-M31-013: queue management: register voter presence; call next by capacity; manage priority sub-queue for elderly, pregnant, and disabled voters; numeric ticket issued on registration
14. FR-M31-014: digital queue tickets visible to DPO with live position and estimated wait time; Presiding Officer calls next via app; priority promotions recorded in audit log
15. FR-M31-015: queue metrics panel: current queue depth, average wait time minutes, throughput per hour, total served count since station opening
16. FR-M31-016: structured incident form with categories ATTEMPTED_DOUBLE_VOTE, BIOMETRIC_FAIL, KIT_ISSUE, PHYSICAL_INCIDENT, VOTER_DISPUTE, ASSISTED_VOTING, OTHER; free-text description; photo and short-video evidence upload with per-file size limit
17. FR-M31-017: escalation chain per incident: to Presiding Officer, to Returning Officer, to county support desk, to war-room; escalation level immutable once set; escalation timestamp recorded and signed
18. FR-M31-018: incident report signed by DPO biometric at submission; queued in Isar + SQLCipher store; submitted to M06 with UUID7 idempotency key on connectivity restoration; duplicate events rejected and not re-counted
19. FR-M31-019: step-by-step paper-backup workflow for KIEMS unavailability: paper voter authentication steps, Form 34A completion guide, spoilt-ballot serial-number logging, subsequent digital reconciliation checklist
20. FR-M31-020: paper-backup form templates embedded offline; no network required at any step; templates only updated via signed MDM push to prevent tampering
21. FR-M31-021: kiosk mode during register-inspection period activates on KIOSK_OPERATOR role credential; enables BVV register lookup (M07) for citizens with staff-assisted OTP retrieval; role validated against MDM enrolment
22. FR-M31-022: kiosk objection filing on behalf of citizen; citizen oral consent, witness biometric, and witness signature captured and stored locally before submission to M07 BVV; consent record retained per DPA 2019 lawful basis
23. FR-M31-023: end-of-day report auto-compiled from local records: opening time, closing time, turnout count, incidents by category, spoilt paper-ballot log with serial numbers, party-agent presence list with arrival and departure times
24. FR-M31-024: end-of-day report requires dual biometric signature from Presiding Officer and DPO; submitted to M06 BVI back-end with UUID7 idempotency key; queued if offline; submission confirmed before session close
25. FR-M31-025: all reference materials (manual, statutory forms, clauses, contact card) fully offline; no dependency on back-end connectivity for any reference function
26. FR-M31-026: incident logs queued in local Isar + SQLCipher store; submitted with UUID7 idempotency key on reconnect; back-end caches response and returns it on re-send; no duplicate audit events
27. FR-M31-027: register-lookup fallback requires network; if offline, lookup queued as pending request with informational status displayed to DPO; DPO notified when lookup completes on reconnect
28. FR-M31-028: battery cost <= 5% per hour foreground; background sync bounded to <= 2% per hour; app respects Android Doze and iOS Background App Refresh limits; foreground-service used for ongoing queue polling only
29. FR-M31-029: mTLS certificate pinning enforced against eel-root CA; captive-portal detection triggers "network blocked" UX; no auth tokens emitted through unpinned connections; pinning failure logged as security event
30. FR-M31-030: screen-capture protection (FLAG_SECURE on Android, UIScreen.isCaptured check on iOS) active on voter-lookup and incident screens; SQLCipher AES-256-GCM on all local Isar stores; key stored in hardware-backed keystore
31. FR-M31-031: WCAG 2.2 AA on every screen; TalkBack and VoiceOver traversal completes all primary flows without dead-end navigation; minimum touch target 48 dp; font scales to 200% without layout break
32. FR-M31-032: offline integrity: Isar WAL journaled before every state mutation; OS-kill at any transition resumable on next launch without data loss or duplicate submission; tested by background-kill harness at each FSM state

## Acceptance criteria

1. AC-M31-01: end-of-day report auto-compiled, dual-signed by Presiding Officer and DPO biometrics, submitted to M06 BVI back-end, and confirmed by audit-trace query; verified in integration test and UAT sign-off with real clerks on real devices
2. AC-M31-02: paper-backup workflow completed end-to-end by a real clerk in field trial with network disabled throughout; digital reconciliation queued and submitted correctly on reconnect; no data lost
3. AC-M31-03: every voter-lookup fallback creates a permanent audit entry in M06 containing DPO biometric attestation hash, timestamp, reason, station code, and lookup result; audit-trace query confirms presence and immutability
4. AC-M31-04: kiosk-mode objection flow reviewed and approved by IEBC Data Protection Officer for DPA 2019 compliance; lawful-basis documentation published and linked in app before kiosk mode activates in production
5. AC-M31-05: all reference materials searchable and accessible in airplane mode; full-text search returns first result in <= 1 s on mid-tier Android 10 device; verified in flutter_test offline benchmark
6. AC-M31-06: MFA sign-in completes on Android 10+ and iOS 15+ real devices; attestation failure on rooted emulator blocks sign-in within 2 s and emits tamper event to M06 audit log
7. AC-M31-07: mTLS pinning refuses connection to MITM proxy presenting rogue CA cert; zero auth-header bytes observed in captive-portal intercept test; verified with mitmproxy captive-mode harness
8. AC-M31-08: incident report with photo evidence queued while offline, transmitted on reconnect, confirmed received by M06 without duplicate; re-send of identical request returns cached 201; idempotency verified in integration test
9. AC-M31-09: priority queue correctly surfaces 48 elderly or disabled synthetic voters ahead of 500 standard-queue entries without manual promotion; ordering verified in flutter_test integration scenario
10. AC-M31-10: 92,000 synthetic clerk check-ins over a 60-minute k6 window processed by back-end with p99 response <= 2 s and zero dropped events; load test run in staging pre-election
11. AC-M31-11: JWT expiry mid-session triggers silent token refresh; in-flight request completes with 200; next request succeeds; user sees no re-authentication prompt; Keycloak 5-second TTL used in test
12. AC-M31-12: WCAG 2.2 AA independent audit returns zero Priority 1 findings; TalkBack primary flow and VoiceOver primary flow complete without dead-end navigation; axe-core scan clean on every screen
13. AC-M31-13: app build produces byte-identical artefacts across three consecutive CI runs; SBOM (SPDX) generated, cosign-signed, and verifiable
14. AC-M31-14: SQLCipher-encrypted Isar store contains no recoverable plaintext PII when APK data partition extracted on rooted device in forensic inspection
15. AC-M31-15: Frida-attached Android device refused at sign-in within 2 s; RASP terminates session and writes tamper event to audit log; verified in security gate using rooted AVD

## Applicable edge cases

**mobile**
- EC-MOB-01: OS kills app mid-sync; local Isar WAL intact; on next launch queue resumes without duplicate submission; background-kill harness exercises every FSM transition
- EC-MOB-02: storage full on device; app refuses new evidence uploads with graceful error; incident text queued; bounded queue size enforced via Isar compaction; verified on filled-disk device
- EC-MOB-03: 1 GB RAM KES-10k device; app heap <= 15 MB; full-text search p99 <= 2 s on 3G; no OOM crash; verified in device-farm memory gate
- EC-MOB-04: Android Doze or iOS background limits suspend background sync; app resumes queue drain on foreground without data loss; no duplicate events
- EC-MOB-05: push token rotated or FCM unavailable; clerk re-registers token silently on next foreground; critical ECC alerts fall back to in-app polling interval
- EC-MOB-06: app version below MDM-enforced floor; update forced within 48 h; local Isar queue survives update; N-1 to N schema migration verified in integration test
- EC-MOB-07: GPS mock-location enabled at check-in; location flagged spoof-suspect; DPO notified; check-in not silently accepted; cell/Wi-Fi delta cross-check applied
- EC-MOB-08: device clock wrong; server-authoritative timestamp used for all audit events; local clock displayed as informational only; unit test confirms no client-clock dependency in signatures
- EC-MOB-09: biometric sensor broken; PIN fallback rate-limited to 5 attempts then 30-minute lockout; no silent policy downgrade; patrol test with sensor-disabled device

**connectivity**
- EC-CON-01: offline >= 48 h; reference materials fully accessible; incident and end-of-day queues durable in Isar; gap-audit submitted on reconnect; proptest confirms queue never drops
- EC-CON-02: 3G 500 kbps / 300 ms RTT; dashboard refresh and check-in submission meet module SLAs; CBOR+zstd negotiated; k6 behind toxiproxy confirms compliance
- EC-CON-03: 2G only; app shows informational "limited connectivity" banner; reference and queue functions fully offline; no data attempted over 2G without user confirmation
- EC-CON-05: flapping link during incident submission; backoff + jitter retry; UUID7 idempotency key prevents duplicate; retry budget exhausted before queue overflows; toxiproxy toggle test
- EC-CON-06: captive portal intercepts TLS; pinning fails closed; "network blocked" UX shown; zero auth tokens emitted; mitmproxy captive-mode test confirms clean
- EC-CON-08: DNS poisoned; hardcoded fallback endpoints used; circuit opens after 3 consecutive failures; clerk sees "using backup connection" notice; DNSChaos test

**identity**
- EC-IDN-02: JWT expires mid-request; in-flight request completes; silent token refresh on next call; no re-authentication prompt shown to clerk; Keycloak 5-second TTL used in Pact test
- EC-IDN-04: device attestation fails (custom ROM or emulator MEETS_NONE); hard-fail blocks sign-in; support code shown for IEBC helpdesk; no partial access granted
- EC-IDN-05: rooted or Frida-attached device detected; RASP terminates session; local SQLCipher keys wiped from memory; tamper event emitted to M06 within 2 s; rooted AVD test
- EC-IDN-06: cloned APK with resigned certificate; TLS pin mismatch closes connection; back-end rejects SVID digest; event emitted to M28 AI monitoring stack
- EC-IDN-07: no SMS OTP used for accredited staff; FIDO2 device-bound credential required; new-device sign-in denied without MDM re-enrolment; ASVS V2 compliance
- EC-IDN-10: SpiceDB unavailable; write operations fail closed with 503 problem+json; read-only class-C dashboard reads continue from last-good cache; PodChaos test
- EC-IDN-11: Keycloak realm key rotated; app refetches JWKS on kid miss; next request succeeds without user action; rotation-in-testcontainer test

**data**
- EC-DAT-01: duplicate incident submission with same UUID7 idempotency key and body; back-end returns cached 201; no duplicate event in eel.field.clerk.v1 topic; auto-generated contract test
- EC-DAT-02: same UUID7 key with different body (tampered retry); back-end returns 422 idempotency-key-reuse; app surfaces conflict error to DPO; auto-generated contract test
- EC-DAT-09: voter names with Unicode characters (Arabic, extended Latin); NFC normalisation applied before hashing; display uses ICU collation; no truncation; hypothesis unicode corpus

**security**
- EC-SEC-05: all free-text inputs (incident description, lookup justification) validated server-side with parameterised queries; no injection path to back-end SQL or event store; Semgrep and ZAP scans clean
- EC-SEC-11: biometric attestation hash comparison uses constant-time compare; no timing oracle on DPO attestation path; dudect-style timing test in CI
- EC-SEC-16: no PII fields (voter ID hash, biometric hash, station code combined with name) appear in structured logs, crash reports, or OTel traces; log-scanner fixture passes in CI

**scale**
- EC-SCL-04: consumer lag on eel.field.clerk.v1 topic exceeds 60 s; back-end pages on-call; backpressure applied to ingest endpoint; app queue bounded to prevent device OOM; no data loss confirmed by slow-consumer chaos test

**human**
- EC-HUM-05: assisted voting scenario; assistant identity recorded in ASSISTED_VOTING incident form; voter consent captured as signed attachment; ballot secrecy maintained; workflow test with PWD UAT
- EC-HUM-06: WCAG 2.2 AA on every screen; TalkBack traversal of queue management and incident form completes without workarounds; axe-core scan and manual walk both pass

**operations**
- EC-OPS-01: KIEMS kit battery dies mid-session; companion app continues functioning on own battery; paper-backup mode activated; end-of-day report queued; journal-before-act ensures exact resume on next app launch

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | lookup rate-limit, queue priority logic, end-of-day arithmetic, paper-backup template rendering | nextest + flutter_test | M31-UNIT- |
| contract | BVI dashboard, BVR lookup, BVV kiosk objection | Pact | M31-PACT- |
| integration | lookup audit trail; incident submission; end-of-day report signing | integration_test | M31-INT- |
| e2e | DPO check-in, lookup, queue, incident, end-of-day | Flutter driver | M31-E2E- |
| security | voter-lookup rate-limit enforcement; kiosk mode ODPC compliance | ZAP | M31-SEC- |
| a11y | WCAG 2.2 AA; searchable reference | axe-core | M31-A11Y- |
| offline | reference materials offline; incident queue offline | netem | M31-OFFLN- |

Tags: `testWidgets('lookup fallback [FR-M31-010,AC-M31-03]')`

## Runbooks and dashboards

- `docs/runbooks/field-mobile-svc-degradation.md` - extend with poll-clerk offline paths and voter-lookup fallback
- Grafana dashboard: `platform/observability/field-mobile-svc/poll-clerk-dashboard.json`
- SLO alerts: `platform/observability/field-mobile-svc/poll-clerk-slo-alerts.yaml`

## Open questions

- ODPC review of kiosk-mode OTP lookup flow: DPA 2019 lawful basis (public task) must be documented before kiosk mode activates; IEBC DPO to lead.
- Paper-backup workflow reconciliation protocol: who owns the digital reconciliation step after paper-backup session? Must be agreed with RO back-office team before field-trial.
