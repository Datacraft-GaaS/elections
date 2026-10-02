# M34 mobile-observer

| field | value |
|---|---|
| M-number | M34 |
| Home service | field-mobile-svc (ADR-0020) |
| Owning work package | WP-4.8 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M34-mobile-observer.md |

## Purpose

Support ~500 accredited observers (civil society, international bodies, academic, media) with a structured mobile tool that mirrors the Public Portal's read views, provides observer-specific incident reporting per accreditation category, lets observers cryptographically verify results at any polling station they visit and countersign verification records, and feeds observer findings to the transparency layer and the post-election Auditor-General report.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Observer App (Android 10+ / iOS 15+) | Flutter | services/field-mobile-svc/apps/observer/ |
| API | Rust (axum) | services/field-mobile-svc/crates/field-mobile-api/ |

Distribution: private via Commission MDM. Languages: EN + SW; international language pack (French, Portuguese) subject to demand.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| ObserverAccreditation | id, observer_id, category (CIVIL_SOCIETY/INTERNATIONAL/ACADEMIC/MEDIA), issuing_body, accreditation_code, valid_from, valid_to, countersign_rights (bool) | B | T1-consensus |
| StationVisitLog | id, observer_id, station_code, check_in_at, check_out_at, gps_lat, gps_lon, observation_note, signed_at, signature | A | T1-audit |
| ObserverIncidentReport | id, observer_id, station_code, category (PROCEDURAL/SECURITY/LEGAL/OPERATIONAL/TAMPERING/TRANSPARENCY/OTHER), description, evidence_blob_ids, gps_lat, gps_lon, is_systemic_flag, submitted_at, signature | B | T1-consensus |
| ObserverVerificationRecord | id, observer_id, station_code, form_id, form_hash, proof_verified (bool), countersign_at, countersignature, audit_trail_id | A | T1-audit |
| EndOfMissionReport | id, observer_id, mission_id, stations_visited, verifications_completed, incidents_reported, generated_at, pdf_blob_id | B | MinIO |

## APIs

OpenAPI file: `packages/openapi-contracts/field/observer.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/observer/portal/results/{station_code} | 3 | JWT+SpiceDB (OBSERVER) | n/a | per-station results with signed Form 34A image |
| GET /v1/observer/portal/results/{station_code}/proof | 3 | JWT+SpiceDB (OBSERVER) | n/a | Merkle-inclusion proof |
| GET /v1/observer/portal/results/bulk/{constituency} | 4 | JWT+SpiceDB (OBSERVER) | n/a | bulk download per constituency (rate-limited) |
| POST /v1/observer/stations/{code}/visit | 3 | JWT+SpiceDB (OBSERVER) | yes | GPS check-in at visited station |
| POST /v1/observer/stations/{code}/visit/sign-off | 3 | JWT+SpiceDB (OBSERVER) | yes | sign off observation session on leaving |
| POST /v1/observer/stations/{code}/verify | 3 | JWT+SpiceDB (OBSERVER) | yes | submit verification record (proof verified client-side) |
| POST /v1/observer/stations/{code}/verify/countersign | 3 | JWT+SpiceDB (OBSERVER) | yes | countersign verification (if accreditation category permits) |
| POST /v1/observer/incidents | 3 | JWT+SpiceDB (OBSERVER) | yes | structured incident report |
| POST /v1/observer/incidents/{id}/systemic | 3 | JWT+SpiceDB (OBSERVER_CHIEF) | yes | flag systemic incident requiring immediate the Commission response |
| GET /v1/observer/end-of-mission | 3 | JWT+SpiceDB (OBSERVER) | n/a | end-of-mission report data |
| GET /v1/observer/end-of-mission/pdf | 3 | JWT+SpiceDB (OBSERVER) | n/a | end-of-mission PDF download |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.field.observer_visit_logged.v1 | emits | eel.field.observer.v1 | operational | yes | at-least-once idempotent |
| eel.field.observer_incident.v1 | emits | eel.field.observer.v1 | operational | yes | at-least-once idempotent |
| eel.field.observer_verification.v1 | emits | eel.field.observer.v1 | operational | yes | at-least-once idempotent |
| eel.field.systemic_incident_flagged.v1 | emits | eel.field.observer.v1 | operational | yes | exactly-once |
| eel.rts.form34a_published.v1 | consumes | eel.rts.form34a.v1 | event-source | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M10 RTS Back-End | internal | integrations/rts/ | yes | Merkle-inclusion-proof retrieval; fallback to CDN cache |
| M11 Public Portal | internal | integrations/portal/ | yes | read-only results data and Form 34A images |
| M13 Public Verifier App | internal | integrations/verifier-lib/ | yes | shared Rust FFI Merkle-verifier; common library |
| M21 RLA Audit Tool | internal | integrations/rla/ | yes | observer verification records feed RLA audit observation stream |
| M29 Mobile Citizen Verify | internal | integrations/verifier-lib/ | yes | shared Rust FFI verifier |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | observer verification and countersignature anchoring |
| Auditor-General escrow mirror | external | integrations/ag-mirror/ | yes (stub) | observer records mirrored for AG post-election audit |

## ML and AI

- None.

## Dependencies

- Modules: M10, M11, M13, M21
- Platform: P05
- Packages: kit-sdk-flutter, eel-crypto-rs, eel-resilience-rs

## Feature checklist

1. FR-M34-001: observer enrolment via Commission accreditation ID + biometric template capture + Play Integrity or App Attest device attestation; attestation failure blocks enrolment; accreditation category recorded and immutable after issue
2. FR-M34-002: MFA sign-in with FIDO2 or platform biometric authenticator + device attestation; fallback to PIN rate-limited at 5 attempts then 30-minute lockout; no SMS OTP for accredited observers
3. FR-M34-003: accreditation category (CIVIL_SOCIETY, INTERNATIONAL, ACADEMIC, MEDIA) determines feature availability at runtime via SpiceDB; only categories with countersign_rights flag receive countersignature UI
4. FR-M34-004: full read of Public Portal (M11) data: national, county, constituency, and station-level results with signed Form 34A images; data fetched from M10 RTS back-end with CDN fallback
5. FR-M34-005: Merkle-inclusion-proof retrieval per station from M10 RTS; client-side verification via shared Rust FFI libeel-verifier crate; verification result displayed within 60 s on LTE
6. FR-M34-006: independent verification of published snapshots against transparency-log (P05) roots; root hash pinned in app bundle at release; mismatch triggers automatic incident report
7. FR-M34-007: bookmark polling stations of interest; bookmarked stations surface in a watch list with push notification when Form 34A published or results updated
8. FR-M34-008: bulk download of signed forms per constituency (rate-limited per accreditation category per hour); download includes Form 34A images and Merkle proofs; progress shown
9. FR-M34-009: GPS check-in on arrival at polling station; distance validated against station polygon; check-in timestamp signed with observer biometric; record submitted to the Commission observer registry
10. FR-M34-010: time-boxed observation session per station; observer signs off with biometric when leaving; session duration and notes recorded; sign-off triggers submission of observation record
11. FR-M34-011: signed observation record (visit log) submitted to the Commission observer registry with UUID7 idempotency key; queued if offline; submitted on reconnect
12. FR-M34-012: structured incident form with 7 categories: PROCEDURAL, SECURITY, LEGAL, OPERATIONAL, TAMPERING, TRANSPARENCY, OTHER; free-text description; photo and short-video evidence upload; GPS coordinates captured at submission
13. FR-M34-013: incident reports AES-256-GCM encrypted locally in Isar + SQLCipher; signed by observer biometric; queued with UUID7 idempotency key; submitted to the Commission observer coordination, war-room topic, and Auditor-General mirror on reconnect
14. FR-M34-014: anonymised aggregated observer-report dashboard visible on Public Portal per the Commission policy; individual reports not linked back to observer identity in public view
15. FR-M34-015: observer chief (OBSERVER_CHIEF role) can signal a systemic incident requiring immediate the Commission response; signal delivered via eel.field.systemic_incident_flagged.v1 exactly-once; SMS/USSD fallback if no network
16. FR-M34-016: guided verification sampling workflow: n stations per constituency selected randomly using cryptographic seed per statistical protocol; protocol version recorded in each verification record
17. FR-M34-017: structured verification checklist per sampled station: form-hash matches transmitted value, party-agent countersignatures present, geo-fence polygon intact, physical seals intact; each item Boolean with notes field
18. FR-M34-018: observer countersignature attached to verification record where accreditation category permits; countersignature anchored to P05 Merkle chain; SpiceDB check enforced at API level
19. FR-M34-019: end-of-mission report auto-compiled from local records: stations visited count, verifications completed, incidents reported by category, duration of field deployment
20. FR-M34-020: end-of-mission report exported to PDF via server-side render; PDF available for download by observer for their own mission-report handoff; PDF hash stored in verification record
21. FR-M34-021: observer verification records feed Auditor-General post-election audit via M21 RLA observation stream; records delivered with signed provenance metadata
22. FR-M34-022: Public Portal snapshots and signed Form 34A images cached in Isar; Merkle-proof verification executable offline against cached root; stale-since timestamp displayed
23. FR-M34-023: station visit logs and incident reports queued in Isar + SQLCipher; submitted on reconnect with UUID7 idempotency keys; no duplicate events on retry
24. FR-M34-024: systemic-incident signal delivered via I08 SMS/USSD fallback (short-code reserved pre-election) if network unavailable; fallback confirmed by IVR ack
25. FR-M34-025: verification cycle per station completes in <= 60 s end-to-end on LTE; battery cost <= 5% per hour foreground; mTLS certificate pinning enforced; biometric unlock required before each submission
26. FR-M34-026: verification hash mismatch detected by Rust FFI verifier; automatic incident report generated with category TAMPERING; station marked "Unverifiable" in app; status never silently cleared without a new successful verification
27. FR-M34-027: screen-capture protection (FLAG_SECURE on Android) active on incident submission screens; SQLCipher AES-256-GCM on all local Isar stores; keys in hardware-backed keystore
28. FR-M34-028: multilingual UI: EN + SW at launch; international language pack (French, Portuguese) loadable on demand via signed MDM or download; language switch persists across sessions
29. FR-M34-029: WCAG 2.2 AA on every screen; TalkBack and VoiceOver traversal completes primary verification and incident flows; minimum touch target 48 dp; font scales to 200%
30. FR-M34-030: offline integrity: Isar WAL journaled before every state mutation; OS-kill at any transition resumable without data loss or duplicate submission; background-kill harness exercises all FSM states
31. FR-M34-031: app build reproducible byte-for-byte across three CI runs; SBOM (SPDX) generated and cosign-verified; MDM private distribution tested on both Android and iOS
32. FR-M34-032: AG mirror integration: observer records mirrored to Auditor-General escrow endpoint with signed provenance within 24 h of submission; delivery confirmed by integrations/ag-mirror/ adapter

## Acceptance criteria

1. AC-M34-01: end-to-end verification of a polling station completed: GPS check-in, checklist filled, Merkle proof verified client-side, countersignature attached (where permitted), session signed off; UAT with accredited observer on real device
2. AC-M34-02: incident report with photo evidence signed, queued offline, transmitted on reconnect, confirmed received without duplicate; re-send of identical request returns cached 201; UAT sign-off
3. AC-M34-03: end-of-mission PDF generated server-side from local records; PDF hash stored in verification record; PDF downloadable by observer; verified in integration test with M21 stub
4. AC-M34-04: Auditor-General mirror receives observer records signed with provenance metadata within 24 h; delivery confirmed by audit-trace query in integrations/ag-mirror/; integration test with AG mirror stub
5. AC-M34-05: French and Portuguese language packs load correctly in EN UI; all user-facing strings translated; date/time formatting uses Kenyan locale; native-speaker sign-off on both packs
6. AC-M34-06: verification hash mismatch on a tampered Form 34A triggers automatic TAMPERING incident report, marks station "Unverifiable", and never silently passes; proptest confirms property over all tampered inputs
7. AC-M34-07: countersignature denied at SpiceDB level to accreditation categories without countersign_rights; attempt returns 403 and is logged; integration test with each category
8. AC-M34-08: MFA sign-in succeeds on Android 10+ and iOS 15+ real devices; attestation failure on rooted emulator blocks sign-in within 2 s and emits tamper event
9. AC-M34-09: mTLS pinning refuses rogue CA cert; zero auth-header bytes in captive-portal intercept test
10. AC-M34-10: Public Portal snapshots verifiable offline; Merkle proof verified against cached root returns correct result in <= 60 s on mid-tier Android without network; airplane-mode integration test
11. AC-M34-11: 500 observers bulk downloading constituency forms simultaneously; rate-limit applied per accreditation category; CDN cache absorbs origin load; k6 simulation confirms p99 <= 2 s
12. AC-M34-12: WCAG 2.2 AA independent audit returns zero Priority 1 findings; TalkBack traversal of verification and incident flows completes; axe-core scan clean on every screen
13. AC-M34-13: no PII (observer ID combined with GPS coordinates, biometric hash) in logs, crash reports, or OTel traces; log-scanner CI fixture passes
14. AC-M34-14: JWT expiry mid-session triggers silent refresh; in-flight request completes; no re-authentication prompt; Keycloak 5-second TTL test
15. AC-M34-15: systemic-incident signal delivered via SMS/USSD fallback when network unavailable; short-code confirmed operational in field trial; IVR ack received within 2 min

## Applicable edge cases

**mobile**
- EC-MOB-01: OS kills app mid-sync; Isar WAL intact; queue resumes on next launch without duplicate submission; background-kill harness exercises every verification FSM state
- EC-MOB-02: storage full on observer device; app refuses new evidence uploads with graceful error; incident text queued; bounded queue via Isar compaction; verified on filled-disk device
- EC-MOB-03: 1 GB RAM KES-10k device; app heap <= 15 MB; Merkle proof FFI call p99 <= 2 s on 3G; no OOM; verified in device-farm memory gate
- EC-MOB-04: Android Doze or iOS background limits suspend sync; cached snapshots remain verifiable offline; queue drains on foreground; no data loss
- EC-MOB-05: push token rotated or FCM unavailable; observer re-registers token silently on next foreground; bookmark notifications fall back to in-app polling
- EC-MOB-06: app version below MDM floor; update forced within 48 h; local Isar queue survives update; N-1 to N schema migration verified
- EC-MOB-07: GPS mock-location enabled at station check-in; check-in flagged spoof-suspect; cell/Wi-Fi delta applied; DPO notified; not silently accepted
- EC-MOB-08: device clock wrong; server-authoritative timestamp used for all visit logs and audit events; local clock shown as informational only
- EC-MOB-09: biometric sensor broken; PIN fallback rate-limited to 5 attempts then lockout; no policy downgrade; patrol test with sensor-disabled device

**connectivity**
- EC-CON-01: observer offline in remote area >= 48 h; Public Portal snapshots and Form 34A images verifiable offline against cached root; incident and visit logs queued; gap-audit on reconnect
- EC-CON-02: 3G 500 kbps / 300 ms RTT; verification cycle completes in <= 60 s; CBOR+zstd negotiated; k6 behind toxiproxy confirms SLA
- EC-CON-03: 2G only; systemic-incident signal falls back to SMS/USSD short-code; app shows "limited connectivity" banner; offline verification remains functional
- EC-CON-05: flapping link during incident submission; backoff + jitter retry; UUID7 idempotency key prevents duplicate; toxiproxy toggle test confirms single event
- EC-CON-06: captive portal intercepts TLS; pinning fails closed; "network blocked" UX shown; no auth tokens emitted; mitmproxy captive-mode test
- EC-CON-08: DNS poisoned; hardcoded fallback endpoints used; circuit opens after 3 failures; observer notified

**identity**
- EC-IDN-02: JWT expires mid-request; in-flight request completes; silent refresh on next call; no re-authentication prompt; Keycloak 5-second TTL test
- EC-IDN-04: device attestation fails (custom ROM); hard-fail blocks sign-in; support code shown; no partial access granted
- EC-IDN-05: rooted or Frida-attached device; RASP terminates session; SQLCipher keys wiped from memory; tamper event emitted within 2 s; rooted AVD test
- EC-IDN-06: cloned APK with resigned cert; TLS pin mismatch; back-end rejects SVID digest; event emitted to M28
- EC-IDN-07: no SMS OTP for accredited observers; FIDO2 device-bound credential required; new device denied without MDM re-enrolment
- EC-IDN-10: SpiceDB unavailable; countersign and write operations fail closed with 503; read-only result view continues from cache; PodChaos test
- EC-IDN-11: Keycloak realm key rotated; JWKS refetched on kid miss; next request succeeds without user action

**data**
- EC-DAT-01: duplicate visit-log submission with same UUID7 key and body; back-end returns cached 201; no duplicate in eel.field.observer.v1 topic
- EC-DAT-02: same UUID7 key with different body; back-end returns 422 idempotency-key-reuse; app surfaces conflict error to observer
- EC-DAT-09: station names with Unicode characters; NFC normalisation before storage; ICU collation in display; no truncation; hypothesis unicode corpus

**security**
- EC-SEC-05: all free-text incident inputs validated server-side with parameterised queries; no injection path; Semgrep and ZAP scans clean
- EC-SEC-11: biometric hash comparison uses constant-time compare; no timing oracle on countersignature path; dudect-style CI test
- EC-SEC-16: no PII (observer ID + GPS, biometric hash) in logs, crash reports, or OTel traces; log-scanner CI fixture passes

**scale**
- EC-SCL-02: 500 observers concurrently bulk-downloading constituency forms; rate-limit per category applied; CDN cache absorbs origin load; STH SDK jitter applied; k6 simulation confirms p99 <= 2 s
- EC-SCL-04: consumer lag on eel.field.observer.v1 topic exceeds 60 s; back-end pages on-call; backpressure applied; app queue bounded; no data loss

**human**
- EC-HUM-06: WCAG 2.2 AA on every screen; TalkBack traversal of verification checklist and incident form completes; axe-core and manual walk both pass
- EC-HUM-08: EN + SW at launch; French + Portuguese language packs downloadable; ICU plurals; font fallback; all strings tested in 150% pseudo-locale

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | accreditation category gate, Merkle proof FFI, countersign rights check, verification sampling protocol | nextest + flutter_test | M34-UNIT- |
| property | Merkle proof always false on tampered form | proptest | M34-PROP- |
| contract | RTS proof API, M11 portal, M21 RLA observation feed, AG mirror | Pact | M34-PACT- |
| integration | full verification cycle; countersign; systemic incident; end-of-mission PDF | integration_test | M34-INT- |
| e2e | observer accreditation, station visit, verify, countersign, incident, end-of-mission report | Flutter driver | M34-E2E- |
| security | attestation fail; countersign denied to wrong category; hash-mismatch auto-incident | ZAP + red team | M34-SEC- |
| a11y | WCAG 2.2 AA; language-pack test | axe-core + manual | M34-A11Y- |
| offline | snapshot verify offline; incident queued | netem | M34-OFFLN- |

Tags: `testWidgets('verify station [FR-M34-005,AC-M34-01,EC-SEC-08]')`

## Runbooks and dashboards

- `docs/runbooks/field-mobile-svc-degradation.md` - extend with observer-app offline paths
- `docs/runbooks/systemic-incident-response.md` - new runbook; war-room escalation flow; the Commission Chair notification steps
- Grafana dashboard: `platform/observability/field-mobile-svc/observer-dashboard.json`
- SLO alerts: `platform/observability/field-mobile-svc/observer-slo-alerts.yaml`

## Open questions

- Auditor-General escrow mirror: AG IT must confirm API or SFTP endpoint for observer record mirroring; timeline and authentication to be agreed.
- Shared Rust FFI Merkle-verifier: same extraction question as M29 and M30; single libeel-verifier crate must be released before any of M29/M30/M34 can ship.
- International language packs (French, Portuguese): translator contracts must be in place if confirmed demand; decision gate at T-120.
- Observer chief systemic-incident path: SMS/USSD fallback via I08; short-code must be reserved and tested before election day.
