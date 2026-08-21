# M05 bvi-device

| field | value |
|---|---|
| M-number | M05 |
| Home service | biometric-edge-svc (ADR-0020) |
| Owning work package | WP-3.8 (docs/plans/04-wave-3-critical-path.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M05-bvi-device.md |

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

1. FR-M05-001: PO opens polling session with biometric auth + timestamp + HSM signature; kit locks to station.
2. FR-M05-002: configurable poll-open/close window per Elections Act; kit enforces window.
3. FR-M05-003: voter auth primary fingerprint; fallback iris -> facial -> MRZ card scan -> name+ID with justification.
4. FR-M05-004: display registered polling station on identification; wrong station -> REDIRECT with correct station name.
5. FR-M05-005: anti-double-voting: mark voter AUTHENTICATED; refuse re-authentication in same session.
6. FR-M05-006: geo-fence enforcement with multi-constellation GNSS + IMU cross-check.
7. FR-M05-007: time-fence enforcement; refuse BVI outside gazetted polling window.
8. FR-M05-008: live on-device dashboard: election name, time vs open/close, turnout %, kit serial, battery, connectivity.
9. FR-M05-009: emergency close per Elections Act s.55 with PO biometric + supervisor biometric (dual-control).
10. FR-M05-010: log every identification event (voter_id, modality, outcome, ts, GPS, kit sig) to local audit + transmit.
11. FR-M05-011: no biometric templates re-transmitted; voter_id + outcome only in events.
12. FR-M05-012: distress finger triggers covert alert; normal auth displayed.
13. FR-M05-013: MRZ fallback after N failed biometric attempts; PO logs justification.
14. FR-M05-014: register pre-load at kit-prep time; refreshed before poll if register updates.
15. FR-M05-015: offline-first; queue transmitted events with retry.
16. FR-M05-016: rejected voter receives printed receipt with reason code and referral.
17. FR-M05-017: audio-guided identification, tactile overlay, high-contrast mode; assistive card lookup.
18. FR-M05-018: party agent read-only view: turnout counters only; no PII; no biometric.
19. FR-M05-019: kit reset/recovery requires dual-control (PO + Deputy PO biometric).
20. FR-M05-020: kit health telemetry every 5 min minimum.
21. Voter auth fingerprint <= 3 s; iris/facial <= 5 s.
22. 700 authentications in <= 10 hours on primary battery + power bank.
23. Kit revocation propagates from central in <= 60 s; kit halts BVI.
24. GNSS jam detected via IMU dead-reckoning; stay operational if IMU agrees; alert if discrepancy.
25. Register extract encrypted on-device; separate key from biometric store.
26. Party agent view rate-limited; no station-level PII exposed.
27. Kit reset event signed and synced to back-end before reset completes.
28. All 11 languages supported in voter-facing screens; WCAG 2.1 AA.

## Acceptance criteria

AC-M05-01: voter identification <= 3 s fingerprint on bench.
AC-M05-02: 700 authentications sustained on battery + power bank (field trial).
AC-M05-03: wrong-station voter correctly redirected with station name (UAT).
AC-M05-04: emergency close requires dual biometric control (policy test).
AC-M05-05: distress finger triggers silent alert within 5 s; UI unchanged (red-team).
AC-M05-06: kit off-fence blocks BVI; back-end alert sent (field trial).
AC-M05-07: GNSS anti-jam mode maintains fence integrity vs IMU (signal test).
AC-M05-08: legacy Android compatibility with documented feature degradation (bench).
AC-M05-09: kit revocation propagates in <= 60 s (chaos exercise).
AC-M05-10: party agent view has no PII exposure (manual + automated scan).
AC-M05-11: zero events lost on power failure at any workflow step (chaos bench).

## Applicable edge cases

**connectivity**
EC-CON-01: 7-day offline queue; ring-buffer signed-sequenced.
EC-CON-02: 3G SLAs met; CBOR+zstd.
EC-CON-05: flapping link; backoff+jitter; server dedup.
EC-CON-09: IPsec tunnel drop mid-batch; resume at sequence.
EC-CON-10: dual-SIM failover; same signed blob.

**time**
EC-TIM-01: clock skew >5 min at boot; refuse session until re-synced.
EC-TIM-02: NTP/NTS unreachable; time_confidence=low.
EC-TIM-07: s.14A close time moves; HSM-signed extension reopens fence.

**identity**
EC-IDN-01: kit cert revoked mid-session; refuse within 60 s.
EC-IDN-08: duress finger; covert alert; UI unchanged.

**data**
EC-DAT-01: duplicate identification event same key; cached response.
EC-DAT-04: replayed signed event; reject on counter.
EC-DAT-09: Unicode voter names; NFC; grapheme-safe.
EC-DAT-11: spoilt > valid totals; block with reason; RO override -> DisputeCase.

**security**
EC-SEC-02: tamper/chassis; kill-on-open; quarantine; ECC alert.
EC-SEC-11: constant-time biometric compare.
EC-SEC-14: stolen kit honey mode; sandbox served.
EC-SEC-16: secrets in logs refused.

**human**
EC-HUM-01: worn fingerprints; multi-finger retry; fallback.
EC-HUM-02: amputee; face/ID path; biometric_exception.
EC-HUM-03: gloves/wet; liveness reject + prompt.
EC-HUM-05: assisted voting; assistant recorded; consent; audio.
EC-HUM-08: 11 languages; ICU plurals.
EC-HUM-11: tactile overlay; >= 48 dp; audio confirm.

**kit**
EC-KIT-01: GNSS jam; multi-constellation + IMU.
EC-KIT-02: legacy Android parity; software attestation tier.
EC-KIT-03: OTA interrupted; A/B slots; rollback.
EC-KIT-04: mode switch not propagated; signed bundle acked.

**operations**
EC-OPS-01: battery death mid-session; journal-before-act; exact resume.
EC-OPS-02: kit swap <= 10 min; inherit roll + watermark; old revoked.
EC-OPS-03: seal broken in transit; custody event; quarantine.
EC-OPS-04: dual-control officer absent; time-boxed ECC remote co-sign.

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

- GNSS spoof risk: multi-constellation + IMU policy needs sign-off from IEBC Security before G3; threshold for "agree vs disagree" between GNSS and IMU fix is not specified in sw/.
- Legacy Android parity matrix: lowest-tier device specs from IEBC ICT needed before feature-matrix test suite can be completed.
