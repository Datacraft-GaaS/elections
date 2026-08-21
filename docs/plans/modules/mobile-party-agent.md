# M30 mobile-party-agent

| field | value |
|---|---|
| M-number | M30 |
| Home service | field-mobile-svc (ADR-0020) |
| Owning work package | WP-4.8 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M30-mobile-party-agent.md |

## Purpose

Give every accredited party agent a phone-in-hand tool that authenticates them into their assigned polling station, lets them independently verify that the Form 34A transmitted from the KIEMS matches the physical form in front of them, countersigns that verification cryptographically via HSM-backed party-agent signature, retrieves and validates constituency and county rollups, and reports incidents to the IEBC war room in a structured, signed way.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Party Agent App (Android 10+ / iOS 15+) | Flutter | services/field-mobile-svc/apps/party-agent/ |
| API | Rust (axum) | services/field-mobile-svc/crates/field-mobile-api/ |

Distribution: private via IEBC MDM (Google Play Managed + Apple Business Manager). Languages: EN + SW.

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

1. FR-M30-001: agent enrols via IEBC accreditation ID + party-issued token + biometric enrolment on device
2. FR-M30-002: assignment bound to specific polling station (or constituency scope for chief agent) per election
3. FR-M30-003: device attestation required; refuses on rooted / jailbroken / attestation-fail devices
4. FR-M30-004: MFA sign-in: biometric + short PIN + device attestation
5. FR-M30-005: agent checks in at assigned polling station; biometric + GPS-verified location
6. FR-M30-006: Presiding Officer receives notification of agent presence via BVI back-end
7. FR-M30-007: party badge display on app for identification to PO
8. FR-M30-008: real-time station dashboard (configurable delay); voters authenticated, turnout %, opening/closing status, kit status; no PII; aggregates only
9. FR-M30-009: aggregate counters only; no voter-level data exposed
10. FR-M30-010: when PO scans Form 34A on KIEMS RTS device, transmission includes QR code visible to agents
11. FR-M30-011: agent scans QR from physical form or from kit screen
12. FR-M30-012: app fetches signed Form 34A record from RTS back-end; displays digital signature chain (PO signature, kit signature, back-end countersignature)
13. FR-M30-013: agent compares displayed values to physical form in front of them
14. FR-M30-014: agent presses "Verify": app fetches Merkle-inclusion proof, validates client-side, confirms transmission integrity
15. FR-M30-015: agent presses "Countersign": biometric + PIN authorises HSM-backed party-agent countersignature attached to form's audit trail
16. FR-M30-016: agent presses "Dispute": flags form as disputed with mandatory reason; escalates to national tally and war room
17. FR-M30-017: Chief Agent view: aggregated Form 34B / 34C as they publish; per-position counts; per-station contribution
18. FR-M30-018: "Verify" button on rollups: fetches Merkle root, validates aggregated hashes
19. FR-M30-019: "Dispute" available at rollup level
20. FR-M30-020: structured incident form: obstruction, tampering, voter intimidation, kit malfunction, illegal campaigning, other; description; evidence upload (photo, short video, audio)
21. FR-M30-021: location auto-captured with consent per-report
22. FR-M30-022: reports encrypted locally; queued; signed by agent; sent to IEBC war room
23. FR-M30-023: report status tracked; IEBC responder can request more info via in-app messaging
24. FR-M30-024: push notifications for: assigned-station events (opening, closing, form submission), chief-agent rollup publication, IEBC broadcast messages
25. FR-M30-025: no PII in push payloads
26. FR-M30-026: offline verification works for cached signed records
27. FR-M30-027: countersignatures and incident reports queued locally; encrypted; signed; submitted on reconnect with idempotency keys
28. Full verification cycle (scan QR + fetch + verify + countersign) <= 30 s
29. Rust FFI signature verifier shared with M29 libeel-verifier
30. Security: duress code triggers silent war-room alert; hooking detection (Frida, Xposed) refuses; screen-capture protection on all screens; local AES-256-GCM encrypted store; no PII in logs

## Acceptance criteria

1. AC-M30-01: end-to-end verify + countersign flow on real device; UAT
2. AC-M30-02: attestation fail refuses correctly; rooted-device test
3. AC-M30-03: countersignature reaches audit ledger with Merkle proof; audit trace
4. AC-M30-04: duress code triggers silent war-room alert; red team test
5. AC-M30-05: incident report end-to-end with attachments; UAT
6. AC-M30-06: offline verification works for cached records; bench test
7. AC-M30-07: MDM force-update policy applied; bench test

## Applicable edge cases

**connectivity**
- EC-CON-01: no network at polling station; cached signed Form 34A verifiable offline; countersignature queued; submitted on reconnect
- EC-CON-03: 2G network; QR scan and proof fetch deferred; 30 s SLA under LTE, extended on 2G

**security**
- EC-SEC-02: GPS spoof attempt by adversary near polling station; multi-constellation + IMU cross-check; spoof alert
- EC-SEC-06: duress code entered by agent under coercion; silent alert to war room; visible flow proceeds normally
- EC-SEC-08: device rooted; attestation hard-fails; app refuses to open

**human**
- EC-HUM-05: agent assists a voter with a dispute; incident report filed with agent biometric; no voter PII in report

**operations**
- EC-OPS-05: wrong-station check-in attempt; GPS distance validation refuses; agent instructed to correct station

**mobile**
- EC-MOB-02: mid-tier Android 10 device; battery <= 5% per hour foreground
- EC-MOB-05: hooking detection; Frida / Xposed refuse
- EC-MOB-06: certificate pinning; mTLS mandatory

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
- MDM enrollment for party-sourced devices: parties supply device list; IEBC MDM enrolls; process must be operationalised at T-60.
