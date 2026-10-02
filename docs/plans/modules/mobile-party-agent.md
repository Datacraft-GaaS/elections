# M30 mobile-party-agent

| field | value |
|---|---|
| M-number | M30 |
| Home service | field-mobile-svc (ADR-0020) |
| Owning work package | WP-4.8 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M30-mobile-party-agent.md |

## Purpose

Give every accredited party agent a phone-in-hand tool that authenticates them into their assigned polling station, lets them independently verify that the Form 34A transmitted from the KIEMS matches the physical form in front of them, countersigns that verification cryptographically via HSM-backed party-agent signature, retrieves and validates constituency and county rollups, and reports incidents to the election authority war room in a structured, signed way.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Party Agent App (Android 10+ / iOS 15+) | Flutter | services/field-mobile-svc/apps/party-agent/ |
| API | Rust (axum) | services/field-mobile-svc/crates/field-mobile-api/ |

Distribution: private via MDM (Google Play Managed + Apple Business Manager). Languages: EN + SW.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| AgentCheckIn | id, agent_id, station_code, election_id, biometric_attestation_hash, gps_lat, gps_lon, check_in_at, signature | B | T1-consensus |
| AgentCountersignature | id, form_id, form_type (34A/34B/34C), agent_id, station_code, merkle_node_id, countersign_at, countersign_type (VERIFIED/DISPUTED), dispute_reason, signature | A | T1-audit |
| AgentIncidentReport | id, agent_id, station_code, incident_type, description, evidence_blob_ids, gps_lat, gps_lon, location_consent, status, submitted_at, signature | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/field/party-agent.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/agent/check-in | 2 | JWT+SpiceDB (PARTY_AGENT) | yes | GPS-verified station check-in; notifies BVI back-end |
| GET /v1/agent/station/{code}/dashboard | 2 | JWT+SpiceDB (PARTY_AGENT) | n/a | aggregate station dashboard (no PII) |
| GET /v1/agent/forms/34a/{id} | 2 | JWT+SpiceDB (PARTY_AGENT) | n/a | fetch signed Form 34A with Merkle proof |
| POST /v1/agent/forms/34a/{id}/countersign | 2 | JWT+SpiceDB (PARTY_AGENT) | yes | attach party-agent countersignature |
| POST /v1/agent/forms/34a/{id}/dispute | 2 | JWT+SpiceDB (PARTY_AGENT) | yes | flag form as disputed with mandatory reason; escalates to war room |
| GET /v1/agent/rollup/34b/{constituency} | 2 | JWT+SpiceDB (PARTY_CHIEF_AGENT) | n/a | Form 34B rollup for chief agent |
| GET /v1/agent/rollup/34c | 2 | JWT+SpiceDB (PARTY_CHIEF_AGENT) | n/a | Form 34C rollup |
| POST /v1/agent/rollup/34b/{constituency}/dispute | 2 | JWT+SpiceDB (PARTY_CHIEF_AGENT) | yes | dispute a rollup |
| POST /v1/agent/incidents | 2 | JWT+SpiceDB (PARTY_AGENT) | yes | submit structured incident report |
| GET /v1/agent/incidents/{id} | 2 | JWT+SpiceDB (PARTY_AGENT) | n/a | incident status |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.field.agent_checked_in.v1 | emits | eel.field.agent.v1 | operational | yes | at-least-once idempotent |
| eel.field.form_countersigned.v1 | emits | eel.field.agent.v1 | operational | yes | at-least-once idempotent |
| eel.field.form_disputed.v1 | emits | eel.field.agent.v1 | operational | yes | exactly-once |
| eel.field.agent_incident.v1 | emits | eel.field.agent.v1 | operational | yes | at-least-once idempotent |
| eel.rts.form34a_captured.v1 | consumes | eel.rts.form34a.v1 | event-source | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M06 BVI Back-End | internal | integrations/bvi/ | yes | check-in notification to PO; circuit breaker; queue |
| M08 RTS Device | internal | integrations/rts-kit/ | yes | Form 34A QR source at kit side |
| M10 RTS Back-End | internal | integrations/rts/ | yes | form retrieval; countersignature submission |
| M11 Public Portal | internal | integrations/portal/ | yes | rollup snapshots (read-only) |
| M23 Grievance Channels | internal | integrations/grievance/ | yes | incident reports escalate through Grievance |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | countersignature anchoring |
| P06 HSM | P06 | integrations/hsm/ | yes | party-agent countersignature via HSM-backed key |

## ML and AI

- None.

## Dependencies

- Modules: M06, M08, M10, M11, M23
- Platform: P05, P06
- Packages: kit-sdk-flutter, eel-crypto-rs, eel-resilience-rs

## Feature checklist

1. FR-M30-001: agent enrols via accreditation ID + party-issued token + biometric enrolment on device
2. FR-M30-002: assignment bound to specific polling station (or constituency scope for chief agent) per election
3. FR-M30-003: device attestation required (hard-fail); refuses on rooted / jailbroken / attestation-fail devices
4. FR-M30-004: MFA sign-in: biometric + short PIN + device attestation; all three required
5. FR-M30-005: agent checks in at assigned polling station; biometric + GPS-verified location; check-in rejected if GPS places agent > 200 m from station
6. FR-M30-006: Presiding Officer receives notification of agent presence via BVI back-end on check-in
7. FR-M30-007: party badge display on app for identification to PO; badge includes party name, agent name, station assignment
8. FR-M30-008: real-time station dashboard (configurable delay); voters authenticated, turnout %, opening/closing status, kit status; no PII; aggregates only
9. FR-M30-009: aggregate counters only; no voter-level data exposed at any point
10. FR-M30-010: when PO scans Form 34A on KIEMS RTS device, transmission includes QR code visible to agents present
11. FR-M30-011: agent scans QR from physical form or from kit screen
12. FR-M30-012: app fetches signed Form 34A record from RTS back-end; displays digital signature chain (PO signature, kit signature, back-end countersignature)
13. FR-M30-013: agent compares displayed values to physical form in front of them
14. FR-M30-014: agent presses "Verify": app fetches Merkle-inclusion proof, validates client-side via Rust FFI verifier, confirms transmission integrity
15. FR-M30-015: agent presses "Countersign": biometric + PIN authorises HSM-backed party-agent countersignature attached to form's audit trail; eel.field.form_countersigned.v1 emitted
16. FR-M30-016: agent presses "Dispute": flags form as disputed with mandatory reason; eel.field.form_disputed.v1 emitted exactly-once; escalates to national tally and war room
17. FR-M30-017: Chief Agent view: aggregated Form 34B / 34C as they publish; per-position counts; per-station contribution
18. FR-M30-018: "Verify" button on rollups: fetches Merkle root, validates aggregated hashes against contributing forms
19. FR-M30-019: "Dispute" available at rollup level; chief agent can dispute 34B or 34C with mandatory reason
20. FR-M30-020: structured incident form: obstruction, tampering, voter intimidation, kit malfunction, illegal campaigning, other; description; evidence upload (photo, short video, audio)
21. FR-M30-021: location auto-captured with consent per-report; no background location collected
22. FR-M30-022: reports encrypted locally in Isar + SQLCipher; queued; signed by agent key; sent to war room on reconnect
23. FR-M30-023: report status tracked; war room responder can request more info via in-app messaging
24. FR-M30-024: push notifications for: assigned-station events (opening, closing, form submission), chief-agent rollup publication, broadcast messages
25. FR-M30-025: no PII in push payloads; content fetched authenticated on receipt
26. FR-M30-026: offline verification works for cached signed records fetched before connectivity loss
27. FR-M30-027: countersignatures and incident reports queued locally; encrypted; signed; submitted on reconnect with idempotency keys; local queue survives phone reboot
28. FR-M30-028: verification cycle (scan QR + fetch + verify + countersign) <= 30 s on LTE; extended SLA on 2G with user notification
29. FR-M30-029: Rust FFI signature verifier shared with M29 libeel-verifier crate; Android JNI + iOS XCFramework; single canonical implementation
30. FR-M30-030: duress code triggers silent war-room alert; eel.field.agent_incident.v1 with type DURESS emitted exactly-once; visible flow continues normally
31. FR-M30-031: hooking detection (Frida, Xposed) at runtime; RASP terminates app; tamper event emitted; MDM notified
32. FR-M30-032: screen-capture protection on all screens; app-switcher preview hidden; OS-level screenshot blocked
33. FR-M30-033: local AES-256-GCM encrypted store (Isar + SQLCipher) with key in device secure enclave; no plaintext at rest
34. FR-M30-034: UUID7 IDs for AgentCheckIn, AgentCountersignature, AgentIncidentReport; generated locally with eel-crypto-rs FFI shim; server validates format
35. FR-M30-035: Riverpod state management; countersignature state immutable after HSM sign; no mutable shared state
36. FR-M30-036: MDM force-update policy: app below floor version blocked on start; MDM notifies agent; local queue survives update

## Acceptance criteria

1. AC-M30-01: end-to-end verify + countersign flow on real device; measurement: UAT sign-off
2. AC-M30-02: attestation fail refuses correctly on rooted device; measurement: rooted-device test with pre-defined test device
3. AC-M30-03: countersignature reaches audit ledger with Merkle proof; measurement: audit trace on T1-audit store
4. AC-M30-04: duress code triggers silent war-room alert; eel.field.agent_incident.v1 DURESS emitted; measurement: red team test
5. AC-M30-05: incident report end-to-end with photo, video, audio attachments reaches war room; measurement: UAT
6. AC-M30-06: offline verification works for cached signed Form 34A; measurement: bench test with netem loss 100%
7. AC-M30-07: MDM force-update policy applied; app below floor version blocked within 60 s of policy push; measurement: bench test
8. AC-M30-08: duplicate countersignature submission same key+body returns cached response without re-execution; measurement: idempotency test
9. AC-M30-09: GPS spoof attempt flagged; check-in refused when cell/Wi-Fi delta > 1 km; measurement: mock-location test
10. AC-M30-10: WCAG 2.2 AA audit pass on all screens; measurement: axe-core + manual audit report
11. AC-M30-11: queue survives phone reboot; countersignatures and incidents resubmit on reconnect without duplication; measurement: reboot + reconnect integration test
12. AC-M30-12: hooking detection refuses Frida attach; RASP terminates within 5 s; measurement: Frida attach test on reference device

## Applicable edge cases

**connectivity**
- EC-CON-01: offline 48 h; cached signed Form 34A verifiable offline; countersignature queued; submitted on reconnect
- EC-CON-02: 3G (500 kbps, 300 ms RTT); verification SLA extended; user notified of delay
- EC-CON-03: 2G only; QR scan and proof fetch deferred; 30 s SLA under LTE; extended on 2G with user notification
- EC-CON-04: site partition primary/mirror; cached form data verifiable offline; queue accumulates until mirror reconnects
- EC-CON-05: flapping link; backoff+jitter; retry budget; no duplicate countersignature submissions
- EC-CON-06: captive portal; mTLS pinning fails closed; "network blocked" UX; no token leak
- EC-CON-07: satellite 600 ms+ RTT; transport-tuned timeouts; MTU-safe
- EC-CON-08: DNS poisoned/down; hardcoded fallback endpoints; circuit opens
- EC-CON-09: IPsec tunnel drop mid-submission; idempotent re-send on reconnect
- EC-CON-10: dual-SIM failover mid-send; same signed blob retried on SIM B
- EC-CON-11: Kafka ISR shrink on eel.field.agent.v1; producer blocks; outbox accumulates; no loss

**identity**
- EC-IDN-01: agent cert revoked mid-session; refuse <= 60 s; honey mode if stolen; queue quarantined
- EC-IDN-02: JWT expires mid-countersign request; in-flight completes; next 401 + silent refresh
- EC-IDN-03: SVID rotates during batch submission; old SVID valid to expiry; producer_key_id per event
- EC-IDN-04: attestation fails (custom ROM); hard-fail; refuse; support code shown
- EC-IDN-05: rooted/emulator/Frida; RASP terminate; wipe; tamper event
- EC-IDN-06: clone app; pin mismatch; backend rejects digest; MDM notified
- EC-IDN-07: SIM swap on staff OTP; no SMS OTP for accredited agent; OTP is device-bound biometric
- EC-IDN-08: duress code entered; silent alert to war room; visible flow continues
- EC-IDN-09: approver revoked mid-countersign dual-control; re-validate both at commit
- EC-IDN-10: SpiceDB down; fail-closed writes; class-C reads only; 503 problem+json
- EC-IDN-11: realm key rotated; stale JWKS; refetch on kid miss

**data**
- EC-DAT-01: duplicate countersignature submission same key+body; cached response; no re-execution
- EC-DAT-02: same countersign key, different body; 422 idempotency-key-reuse

**security**
- EC-SEC-05: injection on all agent API inputs; parameterised only; Semgrep + ZAP + fuzz
- EC-SEC-16: secrets absent from all logs and traces; CI regex gate + log-scanner fixture

**scale**
- EC-SCL-04: consumer lag on eel.field.agent.v1 > 60 s; page alert; backpressure; no OOM

**mobile**
- EC-MOB-01: OS kills mid-sync; resumable; no duplicate; WAL intact
- EC-MOB-02: storage full; graceful refusal; bounded queue; user prompted
- EC-MOB-03: 1 GB RAM mid-tier device; <= 15 MB RAM; no OOM
- EC-MOB-04: Doze/background limits; accredited terminate; resume on foreground
- EC-MOB-05: push token rotation / FCM down; idempotent re-register; SMS for critical alerts
- EC-MOB-06: update mid-election below floor; force floor; 7-day block; queue survives update
- EC-MOB-07: GPS spoof check-in; cell/Wi-Fi delta > 1 km flagged; check-in refused
- EC-MOB-08: device time wrong; server authoritative timestamp used
- EC-MOB-09: biometric sensor broken; PIN fallback rate-limited; no policy downgrade; biometric is required for countersign

**human**
- EC-HUM-05: agent assists a voter with a dispute; incident report filed with agent biometric; no voter PII in report

**operations**
- EC-OPS-05: wrong-station check-in attempt; GPS distance validation refuses; agent instructed to correct station

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | countersignature chain, QR parse, dispute reason validation, GPS-distance check | nextest + flutter_test | M30-UNIT- |
| property | Merkle proof always false on tampered form | proptest | M30-PROP- |
| contract | RTS form retrieval, BVI check-in, Merkle-proof API | Pact | M30-PACT- |
| integration | full countersign flow; dispute escalation; chief-agent rollup verify | integration_test | M30-INT- |
| e2e | agent check-in, Form 34A scan, verify, countersign, incident report | Playwright + Flutter driver | M30-E2E- |
| security | attestation fail; duress code; hooking detection; GPS spoof | ZAP + red team | M30-SEC- |
| a11y | WCAG 2.2 AA; large-text mode | axe-core | M30-A11Y- |
| offline | countersignature queued on reconnect; cache verify | netem | M30-OFFLN- |

Tags: `testWidgets('countersign form [FR-M30-015,AC-M30-03,EC-CON-01]')`

## Runbooks and dashboards

- `docs/runbooks/field-mobile-svc-degradation.md` - extend with agent-app offline countersignature paths
- `docs/runbooks/party-agent-dispute-escalation.md` - new runbook; war-room notification flow; ECC Commander decision steps
- Grafana dashboard: `platform/observability/field-mobile-svc/party-agent-dashboard.json`
- SLO alerts: `platform/observability/field-mobile-svc/party-agent-slo-alerts.yaml`

## Open questions

- Shared Rust FFI Merkle-verifier library (libeel-verifier): ownership and delivery timeline must be agreed with M13/M29 teams before integration sprint; single shared crate.
- HSM-backed party-agent key provisioning: private key provisioning workflow for accredited agents must be designed with P06 team; key-ceremony timing and agent-device binding protocol.
- MDM enrollment for party-sourced devices: parties supply device list; MDM enrolls; process must be operationalised at T-60.
