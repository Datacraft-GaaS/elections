# M34 mobile-observer

| field | value |
|---|---|
| M-number | M34 |
| Home service | field-mobile-svc (ADR-0020) |
| Owning work package | WP-4.8 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M34-mobile-observer.md |

## Purpose

Support ~500 accredited observers (civil society, international bodies, academic, media) with a structured mobile tool that mirrors the Public Portal's read views, provides observer-specific incident reporting per accreditation category, lets observers cryptographically verify results at any polling station they visit and countersign verification records, and feeds observer findings to the transparency layer and the post-election Auditor-General report.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Observer App (Android 10+ / iOS 15+) | Flutter | services/field-mobile-svc/apps/observer/ |
| API | Rust (axum) | services/field-mobile-svc/crates/field-mobile-api/ |

Distribution: private via IEBC MDM. Languages: EN + SW; international language pack (French, Portuguese) subject to demand.

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
| POST /v1/observer/incidents/{id}/systemic | 3 | JWT+SpiceDB (OBSERVER_CHIEF) | yes | flag systemic incident requiring immediate IEBC response |
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

1. FR-M34-001: enrolment via IEBC accreditation ID + biometric + device attestation
2. FR-M34-002: MFA sign-in with FIDO2 or platform authenticator
3. FR-M34-003: accreditation category determines feature availability; only categories with countersign rights get countersignature feature
4. FR-M34-004: full read of Public Portal (M11) data: national, county, constituency, station results with signed Form 34A images
5. FR-M34-005: Merkle-inclusion-proof retrieval and client-side verification via Rust FFI verifier
6. FR-M34-006: independent verification of published snapshots against transparency-log roots
7. FR-M34-007: bookmark stations of interest for follow-along notifications
8. FR-M34-008: bulk download of signed forms per constituency (rate-limited)
9. FR-M34-009: GPS check-in at each polling station visited
10. FR-M34-010: time-boxed observation session; sign off when leaving
11. FR-M34-011: signed observation record submitted to IEBC observer registry
12. FR-M34-012: structured incident form with 7 categories: PROCEDURAL, SECURITY, LEGAL, OPERATIONAL, TAMPERING, TRANSPARENCY, OTHER; description; evidence upload; GPS
13. FR-M34-013: reports encrypted locally; signed; queued; submitted to IEBC observer coordination, war-room, and Auditor-General mirror
14. FR-M34-014: anonymised aggregated observer-report dashboard on Public Portal per IEBC policy
15. FR-M34-015: observer chief can signal a systemic incident requiring immediate IEBC response
16. FR-M34-016: guided verification sampling workflow; n stations per constituency chosen randomly per statistical protocol
17. FR-M34-017: for each sampled station, structured checklist: form-hash matches transmitted, agent countersignatures present, geo-fence intact, seals intact
18. FR-M34-018: observer countersignature attached to verification record (where accreditation permits)
19. FR-M34-019: end-of-mission report auto-compiled: stations visited, verifications completed, incidents reported
20. FR-M34-020: end-of-mission report exported to PDF for observer's own mission report handoff
21. FR-M34-021: feeds Auditor-General post-election audit via M21 observation feed
22. FR-M34-022: Public Portal snapshots cached; verifiable offline
23. FR-M34-023: station visit and incident reports queued; submitted on reconnect
24. FR-M34-024: chief-observer signal delivered via SMS/USSD fallback if network unavailable
25. Verification cycle per station <= 60 s; battery cost <= 5% per hour; mTLS + certificate pinning; biometric unlock required; screen-capture protection on incident screens; AES-256-GCM local store; multilingual EN + SW + French + Portuguese
26. Verification hash mismatch: automatic incident report generated; mark "Unverifiable" not "Verified"

## Acceptance criteria

1. AC-M34-01: end-to-end verification of a polling station; UAT
2. AC-M34-02: incident submission with signed attachments; UAT
3. AC-M34-03: end-of-mission PDF generated; bench test
4. AC-M34-04: Auditor-General mirror receives observer records; audit trace
5. AC-M34-05: multilingual language pack loads for international observers; bench test

## Applicable edge cases

**connectivity**
- EC-CON-01: observer offline in remote area; Public Portal snapshots and cached form data verifiable offline; incident queued
- EC-CON-03: 2G connectivity; chief-observer systemic signal falls back to SMS/USSD

**security**
- EC-SEC-08: verification hash mismatch detected; automatic incident report; "Unverifiable" shown; never silently passed

**legal**
- EC-LEG-04: accreditation category check; countersign right denied to categories without it; hard check in SpiceDB

**mobile**
- EC-MOB-02: mid-tier Android 10; battery <= 5% per hour; verification <= 60 s
- EC-MOB-06: mTLS + certificate pinning; attestation hard-fail

**human**
- EC-HUM-06: WCAG 2.2 AA; accessible incident form
- EC-HUM-08: EN + SW + French + Portuguese; language-pack download before offline operation

**scale**
- EC-SCL-02: 500 observers bulk downloading constituency forms simultaneously; rate-limited; CDN cache absorbs

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
- `docs/runbooks/systemic-incident-response.md` - new runbook; war-room escalation flow; IEBC Chair notification steps
- Grafana dashboard: `platform/observability/field-mobile-svc/observer-dashboard.json`
- SLO alerts: `platform/observability/field-mobile-svc/observer-slo-alerts.yaml`

## Open questions

- Auditor-General escrow mirror: AG IT must confirm API or SFTP endpoint for observer record mirroring; timeline and authentication to be agreed.
- Shared Rust FFI Merkle-verifier: same extraction question as M29 and M30; single libeel-verifier crate must be released before any of M29/M30/M34 can ship.
- International language packs (French, Portuguese): translator contracts must be in place if confirmed demand; decision gate at T-120.
- Observer chief systemic-incident path: SMS/USSD fallback via I08; short-code must be reserved and tested before election day.
