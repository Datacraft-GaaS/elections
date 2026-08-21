# M24 diaspora-voting

| field | value |
|---|---|
| M-number | M24 |
| Home service | biometric-edge-svc (ADR-0020) |
| Owning work package | WP-5.8 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M24-diaspora-voting.md |

## Purpose

Extend BVR and BVI workflows to embassy-hosted environments for Kenyan citizens outside Kenya, providing delayed-batch transmission, embassy-staff-operated kit administration, cross-border RFID chain of custody for physical ballots, and integration of diaspora Form 34A-D results into the national aggregation pipeline.

## Surfaces and languages

| surface | technology | path under services/biometric-edge-svc/ |
|---|---|---|
| Embassy kit (BVR/BVI diaspora mode) | Flutter | apps/diaspora-kit/ |
| Embassy operations dashboard | Phoenix LiveView | ex/diaspora_web/ |
| Physical pouch custody scanner | Flutter | apps/pouch-scanner/ |
| Admin / config API | Rust (axum) | crates/biometric-edge-api/ |
| Public Form 34A-D portal feed | Astro | astro/diaspora-site/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| EmbassyStation | id, embassy_code, host_country_iso3, lat/lon, geofence_radius_km, languages, kit_ids, bvr/bvi windows, status | B | T1-consensus |
| DiasporaVoterRoll | id, embassy_code, roll_version, frozen_at, voter_count, roll_hash, merkle_inclusion_proof | B | T1-consensus |
| DiasporaForm34AD | id, embassy_code, contest_id, tallies, transmission_mode, physical_pouch_rfid, form_hash, merkle_node_id | A | T1-audit |
| BallotPouchCustody | id, pouch_rfid, embassy_code, event_type, custodian_id, lat/lon, event_at, merkle_node_id | A | T1-audit |

## APIs

OpenAPI file: `packages/openapi-contracts/diaspora/diaspora.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/diaspora/embassies | 2 | JWT+SpiceDB | n/a | list embassy stations |
| GET /v1/diaspora/embassies/{code} | 2 | JWT+SpiceDB | n/a | embassy config and status |
| GET /v1/diaspora/embassies/{code}/roll | 2 | JWT+SpiceDB | n/a | current voter roll snapshot |
| POST /v1/diaspora/embassies/{code}/roll/freeze | 2 | JWT+SpiceDB (IEBC_CHAIR + HSM) | yes | freeze and publish embassy roll |
| POST /v1/diaspora/forms/34ad | internal gRPC | SVID mTLS | yes | transmit Form 34A-D from kit |
| GET /v1/diaspora/forms/34ad | 1 | public | n/a | list published diaspora forms |
| GET /v1/diaspora/forms/34ad/{id} | 1 | public | n/a | single form with Merkle proof |
| POST /v1/diaspora/pouch-custody | internal gRPC | SVID mTLS | yes | record pouch custody event |
| GET /v1/diaspora/pouch-custody/{rfid} | 2 | JWT+SpiceDB | n/a | full custody chain for one pouch |
| GET /v1/diaspora/dashboard | 2 | JWT+SpiceDB | n/a | live mission status |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.diaspora.roll_frozen.v1 | emits | eel.diaspora.voter-roll.v1 | operational | yes | at-least-once idempotent |
| eel.diaspora.form34ad_received.v1 | emits | eel.diaspora.form34ad.v1 | event-source | yes | at-least-once idempotent |
| eel.diaspora.pouch_custody_event.v1 | emits | eel.diaspora.ballot-pouch.v1 | operational | yes | at-least-once idempotent |
| eel.diaspora.transmission_delayed.v1 | emits | eel.diaspora.transmission.v1 | operational | yes | at-least-once idempotent |
| eel.diaspora.custody_gap_alert.v1 | emits | eel.diaspora.alert.v1 | operational | yes | at-least-once idempotent |
| eel.rts.form34a_captured.v1 | consumes | eel.rts.form34a.v1 | event-source | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M01 BVR Device / M02 BVR Back-End | internal | integrations/bvr-diaspora/ | yes | circuit breaker; kit queues locally |
| M05 BVI Device / M06 BVI Back-End | internal | integrations/bvi-diaspora/ | yes | circuit breaker; offline mode active |
| M03 ABIS | internal | integrations/abis/ | yes | delayed-batch retry with alert |
| M10 RTS Back-End | internal | integrations/rts/ | yes | circuit breaker; form queued |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | queue; anchor on reconnect |
| P06 HSM | P06 | integrations/hsm/ | yes | signing paused; alert to IEBC Chair |
| M11 Public Portal | internal | integrations/portal/ | yes | fallback to CDN static |
| Foreign Ministry (diplomatic clearance) | external | integrations/foreign-ministry/ | yes (stub) | non-blocking; manual path |
| RFID tracking (M10 shared) | internal | integrations/rfid/ | yes | offline scan queue |
| Airlines / IATA custody | external | integrations/airline-custody/ | yes (stub) | fallback to manual manifest |

## ML and AI

- None in the diaspora-voting module directly; on-device liveness and quality models come from M28 (packaged into diaspora-kit firmware).

## Dependencies

- Modules: M01, M02, M03, M05, M06, M10, M11, M21 (RLA signal via M26)
- Platform: P05 Merkle chain, P06 HSM
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs, kit-sdk-flutter, eel-web-astro

## Feature checklist

1. FR-M24-001: kit provisions in DIASPORA_BVR mode scoped to embassy code and country code
2. FR-M24-002: captures all 22 biographic fields plus host-city, Kenyan ID/Passport, embassy registration date
3. FR-M24-003: 10-finger ISO 19794-2, iris ISO 19794-6, face ICAO 9303 biometric capture; encrypted on-device
4. FR-M24-004: on-device dedup against embassy-local cache; full ABIS dedup via delayed batch
5. FR-M24-005: kit enforces IEBC-gazetted diaspora BVR window; refuses new enrolments outside window
6. FR-M24-006: voter roll freeze 14 days before diaspora voting day; signed snapshot published to IAB and portal
7. FR-M24-007: EN + SW baseline; local-jurisdiction language option for voter-facing screens
8. FR-M24-008: DIASPORA_BVI mode uses frozen embassy roll; no live ABIS sync required
9. FR-M24-009: voting results stored in encrypted signed queue; delayed-batch transmission within window
10. FR-M24-010: DELAYED_TRANSMISSION event if connectivity fails in window; retry every 30 min for up to 6 hours
11. FR-M24-011: physical ballot fallback on digital transmission failure; RFID-tagged tamper-evident pouch
12. FR-M24-012: physical-ballot pouch RFID tracked through embassy dispatch, transit waypoints, JKIA, warehouse, counting centre
13. FR-M24-013: kit runs UTC; displays EAT to IEBC officers, local time to voters
14. FR-M24-014: each kit bound to exactly one embassy code; geo-fence enforced at embassy GPS with 5 km tolerance
15. FR-M24-015: Form 34A-D published within 30 min of digital receipt; within 24 h of Nairobi physical receipt
16. FR-M24-016: diaspora Form 34A-Ds aggregated into dedicated "Diaspora Constituency"; presidential contest only
17. FR-M24-017: all IEBC instructions to presiding officers in EN + SW
18. NFR: kit offline operation for up to 24 hours
19. NFR: encrypted batch over AES-256 + mTLS; certificate pinning
20. NFR: RFID custody gap >= 6 hours auto-triggers IAB alert
21. NFR: diaspora voter roll frozen state reproducible from Merkle root
22. NFR: WCAG 2.2 AA on all voter-facing screens; audio guidance
23. Embassy operations dashboard: live mission list, status badges, transmission-window countdown
24. Voter roll freeze screen: IEBC Chair HSM tap to execute
25. Physical pouch custody scan screen: RFID scanner interface, event type selector, GPS capture
26. Diaspora Form 34A-D public view: per-embassy result card with Merkle proof link
27. Diaspora constituency aggregation: Returning Officer view with Form 34A-D acceptance and Form 34B HSM sign
28. 40-mission concurrent transmission handling in aggregation pipeline
29. Dry-run simulation: full diaspora mission lifecycle (BVR, freeze, BVI, transmission, publication)
30. Geofence enforcement: reject kit operation > 5 km from registered embassy GPS
31. Party-agent custody certificate signed at embassy dispatch
32. IAB alert triggered within 5 min of detecting 6-hour custody gap
33. Per-embassy transmission status published in real time to IEBC Ops dashboard
34. Physical-ballot received confirmed and reconciled against Form 34A-D at counting centre

## Acceptance criteria

1. AC-M24-01: >= 90% of embassy missions achieve digital transmission within declared window; measured by transmission log
2. AC-M24-02: zero physical pouch custody gaps >= 6 hours; measured by RFID log
3. AC-M24-03: diaspora Form 34A-D published <= 30 min of digital receipt; 100% of missions; timestamp delta
4. AC-M24-04: physical-ballot Form 34A-D published <= 24 h of Nairobi receipt; 100%
5. AC-M24-05: voter roll frozen 14 days before diaspora voting day for 100% of missions; freeze timestamp
6. AC-M24-06: diaspora result integrated into national Form 34C; 100%; aggregation log
7. AC-M24-07: kit offline operation sustained for full 8-hour voting day; battery + storage test
8. AC-M24-08: WCAG 2.2 AA independent audit pass on voter-facing kit screens and public portal
9. AC-M24-09: aggregation pipeline handles 40 concurrent embassy transmissions without SLA breach
10. AC-M24-10: roll freeze Merkle root reproducible from signed snapshot; verified by IAB

## Applicable edge cases

**connectivity**
- EC-CON-01: offline 7-day kit queue applies in embassy with no connectivity during BVI session
- EC-CON-02: 3G / 500 kbps link for delayed batch transmission from embassy
- EC-CON-07: satellite RTT > 600 ms for some embassy locations
- EC-CON-09: IPsec tunnel drop mid-batch; resumable at sequence gap

**time**
- EC-TIM-04: diaspora kits span many time zones; UTC storage, gazetted local-time display

**identity**
- EC-IDN-01: kit cert revoked mid-session; refuse <= 60 s
- EC-IDN-08: duress finger during BVI; covert alert, UI identical

**data**
- EC-DAT-01: duplicate Form 34A-D submission; cached response, no re-execution
- EC-DAT-11: spoilt > valid or totals > registered; block with reason

**security**
- EC-SEC-01: signature mismatch on Form 34A-D; never applied; tamper topic
- EC-SEC-02: kit tamper seal broken in transit; quarantine stream; custody event
- EC-SEC-03: HSM unavailable; signing queue; degraded mode

**legal**
- EC-LEG-03: s.44A invoked for voter unable to present biometrics at embassy

**operations**
- EC-OPS-01: kit battery death mid-session; journal-before-act; exact resume
- EC-OPS-02: kit swap <= 10 min; inherit roll; old revoked

**kit**
- EC-KIT-01: GNSS spoof at embassy location; multi-constellation + IMU
- EC-KIT-03: OTA interrupted; A/B slots; rollback

**human**
- EC-HUM-01: worn fingerprints at embassy; multi-finger retry, s.44A fallback
- EC-HUM-08: 11 language strings including Arabic, French for embassy-local language

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | embassy-mode flag enforcement, transmission-window deadline, Form 34A-D schema, custody event FSM | nextest | M24-UNIT- |
| property | RFID chain gap detection, roll-hash reproducibility, offline queue ordering | proptest | M24-PROP- |
| contract | kit-to-backend Form 34A-D submission, RTS aggregation consumer | Pact | M24-PACT- |
| integration | end-to-end diaspora BVR + BVI cycle with simulated embassy kit, delayed-batch on reconnect | testcontainers | M24-INT- |
| e2e | full simulated diaspora mission: registration, roll freeze, voting, digital transmission, portal publication | Playwright + kit-simulator | M24-E2E- |
| offline | 24-hour full-offline BVI; data integrity and transmission on reconnect | netem loss 100% | M24-OFFLN- |
| security | kit cannot activate outside geofence; roll unmodifiable post-freeze; mTLS rotation; ASVS L2 | ZAP + cargo-fuzz | M24-SEC- |
| a11y | WCAG 2.2 AA on voter-facing kit screens and public portal | axe-core + manual | M24-A11Y- |
| load | 40 simultaneous embassy transmission events; aggregation pipeline at peak | k6 | M24-LOAD- |
| chaos | embassy network failure during window; RFID scanner failure at JKIA | Chaos Mesh | M24-CHAOS- |

Tags: `#[trace("FR-M24-001","AC-M24-01")]`

## Runbooks and dashboards

- `docs/runbooks/biometric-edge-svc-degradation.md` - extend with diaspora transmission fallback path
- `docs/runbooks/diaspora-custody-gap-response.md` - new runbook for >= 6-hour custody gap alert
- Grafana dashboard: `platform/observability/biometric-edge-svc/diaspora-dashboard.json`
- SLO alert: `platform/observability/biometric-edge-svc/diaspora-slo-alerts.yaml` (99.9% transmission success)

## Open questions

- How many active missions will be confirmed post-award? Planning estimate is 40, but Foreign Ministry spec exercise (due 2026-12-15) may change the count and affect kit provisioning.
- sw/ spec names Kotlin for kit software; ADR stack mandates Flutter. Page written to ADR-0020 Flutter path; sw/ text should be corrected in the post-award spec exercise.
- Legal opinion per mission cluster on host-country challenge to in-country Kenyan voting is pending; architecture is prepared but activation per mission is discretionary.
- IATA chain-of-custody requirements for physical-ballot pouches need confirmation from IEBC Customs team.
