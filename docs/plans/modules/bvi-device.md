# M05 bvi-device

| field | value |
|---|---|
| M-number | M05 |
| Home service | biometric-edge-svc (ADR-0020) |
| Owning work package | WP-3.8 (docs/plans/04-wave-3-critical-path.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M05-bvi-device.md |

## Purpose

Authenticate every voter presenting at a polling station on election day: confirm registration, prevent double-voting, redirect misdirected voters, transmit polling-day events to the back-end, and never let software failure block the vote as paper backup always exists.

## Surfaces and languages

| surface | technology | path under services/biometric-edge-svc/ |
|---|---|---|
| Kit app (primary) | Flutter on packages/kit-sdk-flutter | apps/bvi-kit/ |
| Legacy Android shim | Flutter + feature-matrix degradation | apps/bvi-kit/ |
| Kit attestation daemon | Rust | crates/biometric-edge-attestation/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| PollingSession | session_id, kit_serial, station_code, po_biometric_hash, opened_at, closed_at, status | C | T1-audit |
| IdentificationEvent | event_id, voter_id (no template), modality, outcome, ts_hlc, gps_fix, kit_sig | C | T1-audit |
| RegisterExtract | extract_id, station_code, voter_list (encrypted), version, signed_at | A | T1-voter (on-device, encrypted) |

## APIs

OpenAPI file: packages/openapi-contracts/bvi/device-events.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /bvi/v1/sessions | internal gRPC | SVID mTLS | yes | PO opens polling session |
| PATCH /bvi/v1/sessions/{id}/close | internal gRPC | SVID mTLS | yes | PO closes session with justification |
| POST /bvi/v1/identifications | internal gRPC | SVID mTLS | yes | Submit identification event batch |
| POST /bvi/v1/distress | internal gRPC | SVID mTLS | yes | Covert distress signal |
| POST /bvi/v1/telemetry | internal gRPC | SVID mTLS | yes | Kit health telemetry |
| GET /bvi/v1/config/{station_code} | internal gRPC | SVID mTLS | n/a | Fetch register extract + config bundle |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.bvi.session_opened.v1 | emits | eel.bvi.session.v1 | operational | yes | at-least-once idempotent |
| eel.bvi.session_closed.v1 | emits | eel.bvi.session.v1 | operational | yes | at-least-once idempotent |
| eel.bvi.voter_identified.v1 | emits | eel.bvi.identification.v1 | operational | yes | at-least-once idempotent |
| eel.bvi.voter_rejected.v1 | emits | eel.bvi.identification.v1 | operational | yes | at-least-once idempotent |
| eel.bve.distress_detected.v1 | emits | eel.bve.kit.v1 | operational | yes | at-least-once idempotent |
| eel.bve.kit_health_reported.v1 | emits | eel.bve.kit.v1 | analytics | yes | at-least-once idempotent |
| eel.bvi.config_bundle_updated.v1 | consumes | eel.bvi.config.v1 | operational | yes | at-least-once idempotent |
| eel.bve.kit_revoked.v1 | consumes | eel.bve.kit.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| BVI back-end (M06) | internal | integrations/bvi-backend/ | yes | queue locally; sync on reconnect |
| BVR register (M02) | internal | integrations/bvr-backend/ | yes | extract loaded at kit-prep; refreshed on demand |
| Geo-fence (M18) | P01 | integrations/boundary-geofence/ | yes | last cached polygon; alert if stale |
| Kit PKI (P04) | I07 | integrations/kit-pki/ | yes | fail closed; kit refuses BVI without valid SVID |
| MDM (O01) | internal | integrations/mdm/ | yes | OTA deferred; current firmware continues |

## ML and AI

- Fingerprint matcher: ISO 19794-2 minutiae match against on-device register extract; no templates transmitted.
- Facial quality gate for fallback path: CNN-based liveness and quality check; no re-identification.

## Dependencies

- Modules: M02 (register extract source), M06 (BVI back-end, event target)
- Platform: P01 (geofence), P04 (kit PKI), P06 (HSM countersign on session close)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-resilience-rs, kit-sdk-flutter, eel-biometric-rs

## Feature checklist

Session management
1. FR-M05-001: PO opens polling session with biometric authentication + timestamp + HSM countersign; kit locks to assigned station code.
2. FR-M05-002: configurable poll-open/close window per Elections Act; kit enforces window via time-fence FSM.
3. FR-M05-003: session close: PO records justification; HSM countersigns session_closed event before transmission.
4. FR-M05-004: emergency close per Elections Act s.55: requires PO biometric + supervisor biometric (dual-control); justification recorded.
5. FR-M05-005: kit reset/recovery: dual-control (PO + Deputy PO biometric); reset event signed and synced to M06 before reset completes.

Voter identification
6. FR-M05-006: voter authentication primary: fingerprint (any of 10 digits); ISO 19794-2 minutiae match against on-device register extract.
7. FR-M05-007: fallback chain: fingerprint > iris > facial > MRZ card scan > name+ID lookup; each step logged with justification and timestamp.
8. FR-M05-008: on successful identification: display voter's registered polling station; name and photo for PO confirmation.
9. FR-M05-009: wrong-station voter: REDIRECT screen with correct station name, GPS map link, and distance.
10. FR-M05-010: anti-double-voting: mark voter AUTHENTICATED in session; refuse re-authentication for same voter in same session.
11. FR-M05-011: rejected voter: print receipt with reason code, referral station (if redirect), referral address, and timestamp.
12. FR-M05-012: MRZ fallback: after N consecutive biometric failures, PO scans ID card MRZ; ICAO 9303 check-digit validated; justification logged.
13. FR-M05-013: no biometric templates re-transmitted; outbound events carry voter_id + modality + outcome only.

Biometric quality gates
14. FR-M05-014: fingerprint quality gate: minimum ISO quality score before match attempt; prompt for retry on poor quality.
15. FR-M05-015: facial liveness gate: CNN-based liveness + quality check before fallback facial comparison; no re-identification.
16. FR-M05-016: worn/faded fingerprint handling: multi-finger retry with adaptive threshold; s.44A-compliant manual fallback.
17. FR-M05-017: amputee path: face/ID path automatically offered; biometric_exception field set; no denial of BVI service.

Anti-replay and audit
18. FR-M05-018: per-event anti-replay: sequence_counter monotonically incremented per kit session; repeated counter rejected as tamper.
19. FR-M05-019: every identification event carries: voter_id, modality, outcome, ts_hlc, gps_fix, kit_serial, sequence_counter, kit_sig.
20. FR-M05-020: local audit log: events written journal-before-act; survives power failure; exact resume on reboot.
21. FR-M05-021: signed events transmitted to M06 via idempotent batch; at-least-once delivery with dedup on event_id.

Distress and security
22. FR-M05-022: distress finger: detect duress biometric template on every fingerprint comparison; covert eel.bve.distress_detected.v1 alert; UI identical to normal auth.
23. FR-M05-023: honey mode: when kit cert is revoked and kit remains powered, serve sandbox auth UI while capturing attacker biometric and alerting war-room.
24. FR-M05-024: SVID mTLS on all kit-to-M06 gRPC calls; SPIFFE URI spiffe://eel.local; no plain-text transport.
25. FR-M05-025: kit certificate managed via SPIRE; invalid or revoked SVID causes kit to refuse BVI operations within 60 s.
26. FR-M05-026: on-device register extract encrypted at rest; AES-256-GCM; key distinct from biometric store key; rotation on reboot.
27. FR-M05-027: ASVS L3 compliance on all kit software components; binary integrity via cosign on every build artifact.
28. FR-M05-028: secrets: no PII, biometric hashes, tokens, or keys in logs, crash dumps, or telemetry; Tier A/B fields rejected.

Register and configuration
29. FR-M05-029: register pre-load at kit-prep time: station-specific voter list encrypted and signed; version tag on extract.
30. FR-M05-030: register refresh: if BVR voter change event arrives before poll open, new extract downloaded and verified before session starts.
31. FR-M05-031: geo-fence enforcement: multi-constellation GNSS + IMU cross-check; kit refuses BVI if fix falls outside station polygon.
32. FR-M05-032: geo-fence polygon update: only via HSM-signed bundle from M06; unsigned polygon rejected; stale polygon alert sent.
33. FR-M05-033: GNSS jam detection: IMU dead-reckoning cross-check; stay operational if IMU and GNSS agree; alert war-room on discrepancy.
34. FR-M05-034: time-fence enforcement: refuse BVI outside gazetted polling window; use NTS/GNSS time; fallback to last-auth monotonic with time_confidence=low.
35. FR-M05-035: clock skew guard: if kit clock skew > 5 min at boot, refuse session until NTS or GNSS time re-established.

Offline and connectivity
36. FR-M05-036: offline-first: ring-buffer queue for transmitted events; 7-day capacity; signed, sequenced, and chunk-resumable.
37. FR-M05-037: IPsec VPN to bvi-svc; reconnect resumes at last acknowledged sequence_counter; idempotent batch re-send.
38. FR-M05-038: dual-SIM failover: same signed event blob transmitted on SIM-B if SIM-A fails mid-batch.
39. FR-M05-039: captive-portal detection: cert pinning fails closed; "network blocked" UX; no auth headers leak.
40. FR-M05-040: 2G fallback: kit refuses 2G data; uses signed SMS/USSD for critical confirmation in <= 5 menu steps.

Dashboards and telemetry
41. FR-M05-041: live on-device dashboard: election name, current time vs open/close, turnout %, voters authenticated, kit serial, battery, connectivity, GPS fix quality.
42. FR-M05-042: kit health telemetry: battery_percent, temperature_celsius, signal_rssi, gps_fix_quality emitted at least every 5 min.
43. FR-M05-043: telemetry alert: war-room receives alert if kit is silent for > 30 min during polling window.
44. FR-M05-044: party agent read-only view: station-level turnout counters only; no PII; no biometric; rate-limited; revocable remotely.

OTA and legacy Android
45. FR-M05-045: OTA firmware update: A/B slots; rollback on failure; cosign-verified bundle; unsigned OTA rejected.
46. FR-M05-046: legacy Android parity: feature-matrix degradation clearly disclosed in app UI on lower-capability devices; parity matrix maintained.
47. FR-M05-047: mode switch: signed config bundle received and acked to M06; unacked switch listed in ECC dashboard; alert after 5 min.

Accessibility and i18n
48. FR-M05-048: audio-guided voter identification flow; tactile overlay; high-contrast mode; >= 48 dp hit targets; assistive Braille card lookup.
49. FR-M05-049: 11 languages on all voter-facing screens; no string truncation; ICU plurals; KSL video explainer for deaf voters; captions.
50. FR-M05-050: colour-blind: all identification outcome indicators use symbol + text + pattern; ΔE >= 3 threshold.
51. FR-M05-051: assisted voting: assistant identity recorded; separate consent captured; no voter secrecy leak; audio confirmation played.

Performance
52. FR-M05-052: fingerprint identification p95 <= 800 ms; p99 <= 1.5 s; iris and facial fallback <= 5 s.
53. FR-M05-053: kit boot from cold start to BVI-ready <= 90 s.
54. FR-M05-054: 700 authentications per 10-hour polling day on primary battery + attached power bank; no battery failure.
55. FR-M05-055: availability election-critical 99.99%; no single software failure blocks voter identification; paper backup path never obstructed.

## Acceptance criteria

AC-M05-01: voter identification fingerprint p95 <= 800 ms; p99 <= 1.5 s on bench (bench test).
AC-M05-02: 700 authentications sustained on primary battery + power bank over 10 hours; no failure (field trial).
AC-M05-03: wrong-station voter correctly redirected with station name and GPS link (UAT).
AC-M05-04: emergency close requires dual biometric control; single-officer close rejected (policy test).
AC-M05-05: distress finger triggers silent alert within 5 s; kit UI shows normal auth to observer (red-team).
AC-M05-06: kit off-fence blocks BVI; war-room alert sent within 30 s (field trial).
AC-M05-07: GNSS anti-jam mode: IMU cross-check maintains fence integrity when GNSS signal is spoofed (signal test).
AC-M05-08: legacy Android compatibility with feature-matrix degradation documented and verified (bench test).
AC-M05-09: kit revocation propagates from M06 in <= 60 s; kit halts BVI on next request (chaos exercise).
AC-M05-10: party agent view contains no PII; automated scan and manual review pass (manual + automated scan).
AC-M05-11: zero identification events lost on power failure at any FSM transition; exact resume on reboot (chaos bench).
AC-M05-12: scan, sign, and transmit identification batch completes <= 90 s per voter at 3G (load test).
AC-M05-13: kit boot from cold start to BVI-ready <= 90 s (bench test).
AC-M05-14: 11 languages render without truncation on all voter-facing screens; WCAG 2.1 AA (i18n + a11y test).
AC-M05-15: 7-day offline ring-buffer: zero events dropped; sequence integrity maintained across simulated 7-day netem loss (proptest).

## Applicable edge cases

connectivity
EC-CON-01: 7-day offline queue: ring-buffer signed-sequenced; sync with gap audit on reconnect; netem loss 100% test.
EC-CON-02: 3G (500 kbps, 300 ms RTT): BVI identification SLAs met; CBOR+zstd negotiated for event batches.
EC-CON-03: 2G only: kit refuses 2G data path; signed SMS/USSD fallback for critical voter confirmation in <= 5 steps.
EC-CON-05: flapping link: backoff+jitter retry; idempotency key prevents duplicate identification event.
EC-CON-06: captive portal: cert pinning fails closed; "network blocked" shown on kit; no auth headers leak.
EC-CON-07: satellite 600 ms+ RTT: transport-tuned timeouts; MTU-safe chunk sizes; SMS compression for critical acks.
EC-CON-09: IPsec tunnel drop mid-batch: resume at last acknowledged sequence_counter; idempotent re-send.
EC-CON-10: dual-SIM failover mid-send: same signed blob transmitted on SIM-B; exactly one identification event lands.

time
EC-TIM-01: kit clock skew > 5 min at boot: session refused until NTS/GNSS time re-established; FSM tested with faked RTC.
EC-TIM-02: NTP/NTS unreachable all day: last-auth monotonic used; time_confidence=low in all events; tolerance window applied.
EC-TIM-04: DST/TZ on diaspora kits: all timestamps stored in UTC; gazetted EAT displayed; close computed in UTC; TZ snapshot test.
EC-TIM-07: s.14A close time moves 17:00 to 19:00: HSM-signed time-extension bundle reopens time fence; forged extension rejected.

identity
EC-IDN-01: kit cert revoked mid-session: BVI refused on next request within 60 s; honey mode if kit is stolen; queue quarantined.
EC-IDN-02: SVID expires mid-request: in-flight identification completes; next gRPC call gets 401 + SPIRE token refresh.
EC-IDN-08: duress finger: duress template matched on fingerprint comparison; covert ECC alert; UI identical to normal auth.
EC-IDN-10: SpiceDB down: kit-admin writes fail closed; class-C register reads served from cached decisions.

data
EC-DAT-01: duplicate identification event same key+body: cached 200 returned; no double-mark of voter; proptest verified.
EC-DAT-02: same idempotency key, different body: 422 idempotency-key-reuse; kit retransmits original event.
EC-DAT-04: replayed signed event with stale sequence_counter: rejected; eel.bvi.tamper.v1 emitted.
EC-DAT-09: Unicode voter names: NFC normalisation on display; ICU collation for name+ID lookup fallback; grapheme-safe.
EC-DAT-11: spoilt > valid voter counts at station: block with reason; RO override creates DisputeCase; TLA+ invariant checked.

security
EC-SEC-02: tamper/chassis seal broken: kill-on-open triggered; quarantine event stream; ECC alert; custody event logged.
EC-SEC-05: injection on all input fields (ID number, name): parameterised only; Semgrep compile-time; fuzz corpus.
EC-SEC-11: timing side-channel on biometric compare: constant-time compare enforced; dudect-style test; Semgrep ban naive ==.
EC-SEC-14: stolen kit honey mode: revoked cert triggers sandbox UI; attacker biometric captured for forensics.
EC-SEC-16: secrets in logs/crash dumps: telemetry refuses Tier A/B fields; log-scanner fixture in CI; no biometric hash in traces.

human
EC-HUM-01: worn/faded fingerprints: multi-finger retry with adaptive quality threshold; s.44A-compliant fallback offered.
EC-HUM-02: amputee: face/ID path automatically offered; biometric_exception field set; no denial of BVI service.
EC-HUM-03: gloves/wet/dusty fingers: liveness reject + prompt to clean; latency budget kept; retry without delay.
EC-HUM-04: low-light or direct sun: auto-exposure, torch prompt, perspective correction, quality gate before facial capture.
EC-HUM-05: assisted voting: assistant identity recorded; separate consent captured; no voter secrecy leak; audio confirmation.
EC-HUM-06: screen readers: full BVI task completion at WCAG 2.1 AA; axe + TalkBack scripts pass.
EC-HUM-07: deaf / KSL: KSL video explainer for voter identification flow; captions present and auto-verified.
EC-HUM-08: 11 languages, long strings: no truncation at 150% pseudo-locale; ICU plurals; font fallback stack.
EC-HUM-09: colour-blind: identification outcome symbols + text + pattern; ΔE >= 3 contrast threshold.
EC-HUM-10: low-literacy USSD: numeric confirmation in <= 5 steps for connectivity-loss path.
EC-HUM-11: Braille/tactile overlay: >= 48 dp calibrated hit targets; audio confirmation of outcome.
EC-HUM-12: clerk fatigue wrong column: ICR cross-check on name+ID fallback; low-confidence score forces re-entry.

kit
EC-KIT-01: GNSS jam/spoof: multi-constellation + IMU cross-check; refuse BVI outside fence if IMU also disagrees.
EC-KIT-02: legacy Android parity: software attestation tier applied; policy-marked; parity matrix test via Pact.
EC-KIT-03: OTA interrupted: A/B slot rollback; signed bundle only; unsigned OTA rejected before flash.
EC-KIT-04: mode switch not propagated: signed bundle must be acked; unacked switch listed in ECC dashboard; 5-min alert.

operations
EC-OPS-01: battery death mid-session: journal-before-act on every FSM step; exact resume on power restore; no double-token.
EC-OPS-02: kit swap <= 10 min: replacement inherits roll + watermark; old kit revoked; event streams linked.
EC-OPS-03: seal broken in transit: custody event logged; kit quarantined; war-room alert sent.
EC-OPS-04: dual-control officer absent: time-boxed ECC remote co-sign for emergency close; logged.
EC-OPS-06: flood/curfew relocation: geo-fence polygon updated only via HSM-signed bundle; forged update rejected; portal notice.

scale
EC-SCL-04: consumer lag > 60 s on bvi event topics: SRE paged; backpressure applied to kit transmit queue; no OOM.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | time-fence FSM, geo-fence containment, anti-double-voting, distress matcher, fallback modality chain | nextest + Flutter test | M05-UNIT- |
| property | queue never drops; double-auth impossible; clock-skew FSM correctness | proptest | M05-PROP- |
| contract | bvi-backend session and identification endpoints; register extract feed | Pact | M05-PACT- |
| integration | offline queue drain; geo-fence double; register extract load | nextest + testcontainers | M05-INT- |
| e2e | full voter identification on emulator and real kit; fallback modality chain | patrol + HIL bench | M05-E2E- |
| fuzz | identification payload; MRZ; GPS coordinate injection | cargo-fuzz | M05-FUZZ- |
| load | 700 auth/day; peak 120 auth/hr; queue drain throughput | k6 + HIL bench | M05-LOAD- |
| chaos | power cut at every FSM step; cert revocation; GNSS kill; OTA interrupt | netem + kill -9 | M05-CHAOS- |
| security | OWASP Mobile Top 10; binary integrity; Semgrep; cert-pin bypass; GNSS spoof | OWASP + Semgrep | M05-SEC- |
| a11y | WCAG 2.1 AA; TalkBack; 48 dp hit targets; audio-guided | axe + TalkBack | M05-A11Y- |
| i18n | 11 locales; 150% pseudo; voter name Unicode | Flutter intl | M05-I18N- |

## Runbooks and dashboards

- docs/runbooks/biometric-edge-svc-degradation.md: extends M01 runbook with BVI-specific fallback chain and session-recovery procedure.
- docs/runbooks/bvi-device-party-agent.md (create): party agent view activation; PII-exposure verification procedure.
- platform/observability/biometric-edge-svc/grafana-bvi-kit.json: identification rate, modality distribution, queue depth, battery, GPS fix status.
- platform/observability/biometric-edge-svc/slo-alerts.yaml: 99.99% burn rate; auth latency breach; revocation propagation breach.

## Open questions

- GNSS spoof risk: multi-constellation + IMU policy needs sign-off from the Commission Security before G3; threshold for "agree vs disagree" between GNSS and IMU fix is not specified in sw/.
- Legacy Android parity matrix: lowest-tier device specs from the Commission ICT needed before feature-matrix test suite can be completed.
