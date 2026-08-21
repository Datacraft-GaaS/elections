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
| POST /v1/diaspora/embassies/{code}/roll/freeze | 2 | JWT+SpiceDB (COMMISSION_CHAIR + HSM) | yes | freeze and publish embassy roll |
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

Registration (BVR diaspora mode):

1. FR-M24-001: Kit provisioned in DIASPORA_BVR mode; voter roll scoped to embassy-jurisdiction-registered Kenyans; restricted by embassy code and diplomatic-mission country code; cannot activate outside configured scope.
2. FR-M24-002: Capture same 22 biographic fields as M01 plus: host-country city of residence, Kenyan ID or Passport number (mandatory), embassy registration date.
3. FR-M24-003: Biometric capture: 10 fingerprints ISO/IEC 19794-2, iris ISO/IEC 19794-6, face ICAO 9303; all biometric data encrypted on-device; synced to central ABIS via encrypted delayed batch (not live streamed).
4. FR-M24-004: On-device dedup against embassy-local cache during session; full ABIS dedup via delayed sync; dedup result published and confirmed before voter roll frozen.
5. FR-M24-005: Embassy registration window enforced by kit; refuses new BVR enrolments outside IEBC-gazetted diaspora window; diaspora window dates gazetted separately from domestic dates.
6. FR-M24-006: Voter roll freeze executed 14 days before diaspora voting day; signed snapshot published to IAB and public portal; requires IEBC Chair HSM card tap to execute; freeze is irreversible without a new signed bundle.
7. FR-M24-007: Multi-language: EN + SW baseline on all screens; local-jurisdiction language as additional display option for voter-facing screens (Arabic for Middle East missions, French for Francophone Africa missions).

Voting (BVI diaspora mode):

8. FR-M24-008: DIASPORA_BVI mode uses frozen embassy roll; double-vote check performed against embassy-local signed roll snapshot; no live central ABIS sync required.
9. FR-M24-009: Voting results stored in encrypted signed queue on-device; transmission scheduled within IEBC-published per-embassy transmission window (published 30 days before voting day).
10. FR-M24-010: Transmission window management: on connectivity failure, kit generates DELAYED_TRANSMISSION event; data retained under continuous party-agent custody; retransmission attempted every 30 minutes for up to 6 hours after poll close.
11. FR-M24-011: Physical ballot fallback: on digital transmission failure after 6-hour retry window, physical ballots sealed in RFID-tagged tamper-evident pouch dispatched to Nairobi under IEBC observer escort; party-agent custody certificate signed at embassy dispatch.
12. FR-M24-012: Cross-border chain of custody: physical-ballot pouch RFID custody events recorded at embassy dispatch, transit waypoints, JKIA arrival, bonded warehouse, and Nairobi counting centre; all events anchored to Merkle chain.
13. FR-M24-013: Time zone handling: kit runs UTC; displays EAT to IEBC officers, local time to voters; transmission window published in both UTC and local time for each embassy.
14. FR-M24-014: Kit bound to exactly one embassy code; cannot operate at a different embassy; geo-fence enforced at embassy GPS coordinates with 5 km tolerance for large-venue voting days.
15. FR-M24-015: Form 34A-D publication: digitally transmitted form published on public portal within 30 minutes of central receipt; physical-ballot form published within 24 hours of Nairobi receipt; 100% of missions.
16. FR-M24-016: Diaspora results aggregated into dedicated "Diaspora Constituency" in national aggregation pipeline; presidential contest only (diaspora voters do not participate in constituency contests under current Elections Act framework).
17. FR-M24-017: All IEBC instructions to embassy presiding officers published in EN + SW; embassy-country liaison language at IEBC discretion.

Availability and offline:

18. FR-M24-018: Kit offline operation for up to 24 hours; BVR and BVI workflows unaffected by network loss; local encrypted queue holds all data.
19. FR-M24-019: Encrypted batch sync over AES-256 with mTLS; certificate pinning to Kenyan cloud endpoint; no transmission without valid pinned certificate.
20. FR-M24-020: Physical-ballot pouch RFID tracked; any custody gap >= 6 hours triggers automatic IAB alert within 5 minutes of gap detection.
21. FR-M24-021: Diaspora voter roll frozen state reproducible from Merkle root; any post-freeze modification is detectable by roll-hash comparison.
22. FR-M24-022: WCAG 2.2 AA for all voter-facing screens; audio guidance available on kit and public portal.
23. FR-M24-023: Multi-language voter-facing: EN + SW + local-jurisdiction language where configured per embassy station.
24. FR-M24-024: DPA 2019 compliance: biometric data of diaspora voters processed under same framework as domestic voters (DPA 2019 s.26); cross-border transfer to Kenyan ABIS falls within Kenyan controller jurisdiction; no third-country adequacy required.
25. FR-M24-025: Diaspora aggregation backend available >= 99.9% during per-embassy transmission windows.
26. FR-M24-026: Kit provisioning for up to 60 diplomatic missions; post-award specification exercise confirms final kit count and embassy languages per mission.
27. FR-M24-027: All diaspora events anchored to P05 Merkle chain within 60 minutes of event or on sync if offline.

APIs:

28. FR-M24-028: GET /v1/diaspora/embassies: JWT+SpiceDB; list all embassy stations with status.
29. FR-M24-029: GET /v1/diaspora/embassies/{code}: JWT+SpiceDB; embassy config, kit list, and current status.
30. FR-M24-030: GET /v1/diaspora/embassies/{code}/roll: JWT+SpiceDB; current voter roll snapshot with hash.
31. FR-M24-031: POST /v1/diaspora/embassies/{code}/roll/freeze: COMMISSION_CHAIR + HSM; idempotent write; one freeze per embassy; irreversible.
32. FR-M24-032: POST /v1/diaspora/forms/34ad: internal gRPC; SVID mTLS; idempotent write; form transmitted from kit.
33. FR-M24-033: GET /v1/diaspora/forms/34ad: public; list all published diaspora forms.
34. FR-M24-034: GET /v1/diaspora/forms/34ad/{id}: public; single Form 34A-D with Merkle proof.
35. FR-M24-035: POST /v1/diaspora/pouch-custody: RFID scanner cert; idempotent write; custody event from scanner.
36. FR-M24-036: GET /v1/diaspora/pouch-custody/{rfid}: JWT+SpiceDB; full custody chain for one pouch.
37. FR-M24-037: GET /v1/diaspora/dashboard: COMMISSION_OPS JWT; live mission status with transmission-window countdowns.
38. FR-M24-038: POST /v1/diaspora/alert/custody-gap: internal system endpoint; auto-triggered on >= 6-hour gap; idempotent.

Events:

39. FR-M24-039: Emit eel.diaspora.roll_frozen.v1 on roll freeze; signed; at-least-once idempotent; consumed by IAB audit stream.
40. FR-M24-040: Emit eel.diaspora.form34ad_received.v1 on form receipt from kit; signed; at-least-once; triggers publication pipeline.
41. FR-M24-041: Emit eel.diaspora.pouch_custody_event.v1 on each RFID scan; signed; at-least-once idempotent.
42. FR-M24-042: Emit eel.diaspora.transmission_delayed.v1 when digital transmission not completed before window close; signed; triggers physical-ballot workflow.
43. FR-M24-043: Emit eel.diaspora.custody_gap_alert.v1 when RFID gap >= 6 hours; signed; exactly-once; consumed by IAB alert queue.

UI screens:

44. FR-M24-044: Embassy operations dashboard: list of all missions; status badge (green/amber/red) per mission; transmission-window countdown; transmission mode indicator (digital/physical/delayed); one-click escalation to IEBC Ops Officer.
45. FR-M24-045: Diaspora BVR Wizard on-kit: identical UX to M01 registration; embassy-specific header; language toggle with local-jurisdiction language option; confirmation screen shows embassy code prominently.
46. FR-M24-046: Voter roll freeze screen: IEBC Chair view; roll count, pending dedup status, freeze timestamp preview; HSM card tap required; confirmation dialog before execution.
47. FR-M24-047: Diaspora BVI flow on-kit: identical to M05 BVI UX; offline mode banner showing "offline, voting will be transmitted within [window close time]" when no connectivity.
48. FR-M24-048: Physical pouch custody screen: RFID scanner interface; custody event type selector; party-agent witness QR scan; automatic GPS capture; confirmation before record submission.
49. FR-M24-049: Diaspora Form 34A-D public view: per-embassy result card with candidate tallies, transmission mode, timestamp, Merkle proof link; identical styling to domestic Form 34A view on public portal.
50. FR-M24-050: Diaspora constituency aggregation: Returning Officer view; all embassy Form 34A-Ds with acceptance status; aggregate Form 34B display; HSM-sign button; disabled until all missions in CLOSED or PHYSICAL_BALLOT_RECEIVED state.

Integration and logistics:

51. FR-M24-051: Foreign Ministry diplomatic clearance integration: kit export authorisation request; stub in dev; non-blocking on kit activation; manual fallback path documented.
52. FR-M24-052: Airlines/IATA custody integration for physical-ballot pouches: RFID events cross-referenced with air-freight manifest; stub in dev; fallback to manual manifest upload.
53. FR-M24-053: Physical-ballot received at Nairobi counting centre: RFID scan triggers COUNT_CENTRE_RECEIPT event; reconciliation check against Form 34A-D registered voter count before Form 34A-D is published.

## Acceptance criteria

AC-M24-01: >= 90% of embassy missions achieve digital transmission within the declared per-embassy window; measured by transmission log signed by IEBC Diaspora Ops.
AC-M24-02: Zero physical pouch custody gaps >= 6 hours across all missions on election day; measured by RFID event log; IAB Diaspora Representative sign-off.
AC-M24-03: Diaspora Form 34A-D published <= 30 minutes of digital receipt for 100% of missions; timestamp delta logged; Datacraft Technical sign-off.
AC-M24-04: Physical-ballot Form 34A-D published <= 24 hours of Nairobi counting-centre receipt for 100% of missions; Returning Officer sign-off.
AC-M24-05: Voter roll frozen 14 days before diaspora voting day for 100% of missions; freeze timestamp verified against gazette date per mission.
AC-M24-06: Diaspora result integrated into national Form 34C for 100% of missions; aggregation log verified by Returning Officer (Diaspora).
AC-M24-07: Kit offline operation sustained for full 8-hour voting day on all provisioned kit models; battery and storage tests pass; 100% of missions.
AC-M24-08: WCAG 2.2 AA independent audit pass on voter-facing kit screens and diaspora public portal; screen-reader full-task-completion verified.
AC-M24-09: Aggregation pipeline handles 40 concurrent embassy transmissions without SLA breach; load test confirms no Form 34A-D delayed beyond 30-minute window.
AC-M24-10: Roll freeze Merkle root reproducible from signed snapshot; IAB Diaspora Representative verification confirms hash equality.
AC-M24-11: Kit cannot activate outside configured geofence (> 5 km from registered embassy GPS); security test with spoofed GNSS coordinates confirms refusal.
AC-M24-12: IAB alert triggered within 5 minutes of detecting >= 6-hour RFID custody gap; chaos test confirms alert delivery even under network degradation.
AC-M24-13: Diaspora aggregation backend available >= 99.9% during per-embassy transmission windows; SLO burn-rate alert fires before budget exhausted.
AC-M24-14: DELAYED_TRANSMISSION event generated on window expiry; retransmission attempted every 30 minutes; physical-ballot workflow initiated after 6-hour maximum; integration test verifies full state machine.

## Applicable edge cases

EC-CON-01: Offline 7-day kit queue applies when embassy has no connectivity during BVI session; ring-buffer signed+sequenced; sync with gap audit on reconnect; no ballot data lost.
EC-CON-02: 3G/500 kbps link for delayed batch transmission from embassy; CBOR+zstd compression negotiated; module 3G SLAs met.
EC-CON-07: Satellite RTT > 600 ms at remote embassy locations (Pacific, South America); transport-tuned timeouts; MTU-safe packet sizing.
EC-CON-09: IPsec tunnel drop mid-batch during Form 34A-D transmission; resume at sequence gap; idempotent re-send; no duplicate form record.
EC-CON-10: Dual-SIM failover mid-transmission at embassy; same signed batch retransmitted on SIM B; single form record ensured.
EC-DAT-01: Duplicate Form 34A-D submission from kit on reconnect; cached response returned; no re-execution of aggregation pipeline.
EC-DAT-02: Same idempotency key with different Form 34A-D body (e.g., corrected tally attempt); 422 idempotency-key-reuse error; Returning Officer must use amendment flow.
EC-DAT-11: Spoilt > valid or candidate tallies > registered voters in embassy form; block with reason code; Returning Officer override creates DisputeCase.
EC-SEC-01: Signature mismatch on Form 34A-D received from kit; form never applied to aggregation; tamper topic event emitted; alert to SOC + IEBC Chair.
EC-SEC-02: Kit tamper seal broken in transit between embassy and counting centre; quarantine stream; RFID custody event raised; ECC alert triggered.
EC-SEC-03: HSM unavailable during roll-freeze ceremony; signing queue pauses; alert to IEBC Chair; freeze postponed; ingest continues.
EC-SEC-05: Injection in embassy station metadata field submitted via admin API; parameterised storage only; no dynamic query.
EC-SEC-16: Biometric data or voter ID leaked to log lines; CI log-scanner fixture blocks deployment; telemetry refuses Tier A/B data.
EC-IDN-01: Kit cert revoked mid-session due to suspected compromise; kit refuses further operations within 60 s; honey mode served; queue quarantined.
EC-IDN-02: JWT expires mid Returning Officer aggregation action; in-flight request completes; next request returns 401 with silent refresh.
EC-IDN-08: Duress finger during diaspora BVI; covert ECC alert transmitted; UI identical for voter; session continues normally.
EC-IDN-10: SpiceDB down during roll-freeze attempt; fail closed on freeze write; dashboard read-only continues with cached data.
EC-TIM-04: Diaspora kits span UTC+3 to UTC-8 time zones; UTC storage enforced; gazetted local-time display per embassy; close computed in UTC.
EC-SCL-04: Kafka consumer lag > 60 s on form34ad received topic; page alert triggered; backpressure applied; no OOM on aggregation worker.
EC-LEG-03: s.44A invoked at embassy for voter unable to present biometrics (injury, sedation); dual party-agent witness required; reason code MEDICAL_INABILITY logged.
EC-OPS-01: Kit battery death mid-session at embassy; journal-before-act pattern; exact resume at last authenticated voter step; no double ballot issued.
EC-OPS-02: Kit swap at embassy in <= 10 minutes; inherit frozen embassy roll and watermark; old kit revoked; custody streams linked.
EC-KIT-01: GNSS spoof at embassy location; multi-constellation cross-check + IMU validation; kit refuses operation if geofence mismatch confirmed.
EC-KIT-03: OTA update interrupted during embassy pre-deployment; A/B firmware slots; automatic rollback to previous known-good version.
EC-HUM-01: Worn fingerprints at embassy due to manual labour; multi-finger retry with adaptive quality threshold; s.44A fallback with reason code and party-agent witness.
EC-HUM-08: 11 language strings including Arabic and French for embassy-local language display; no truncation; font fallback stack tested.

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
