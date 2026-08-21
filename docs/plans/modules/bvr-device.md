# M01 bvr-device

| field | value |
|---|---|
| M-number | M01 |
| Home service | biometric-edge-svc (ADR-0020) |
| Owning work package | WP-3.8 (docs/plans/04-wave-3-critical-path.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M01-bvr-device.md |

## Purpose

Capture biographic and biometric data for every citizen registering to vote, operating offline-first on the KIEMS kit and legacy Android devices. Guarantees zero record loss or tampering between capture and BVR back-end acceptance.

## Surfaces and languages

| surface | technology | path under services/biometric-edge-svc/ |
|---|---|---|
| Kit app (primary) | Flutter on packages/kit-sdk-flutter | apps/bvr-kit/ |
| Legacy Android shim | Flutter + feature-matrix degradation layer | apps/bvr-kit/ |
| Kit attestation daemon | Rust crate | crates/biometric-edge-attestation/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| Voter | voter_id, biographic_fields (22), status, version | A | T1-voter |
| BiometricTemplate | template_id, voter_id, modality, iso_format, wsq_blob, encrypted | A | T1-voter (field-encrypted) |
| IDDocumentScan | scan_id, voter_id, mrz_parsed, pdf_hash, encrypted_pdf_ref | A | MinIO |
| Signature | sig_id, voter_id, stylus_vector_hash | A | T1-voter |
| AuditEvent | event_id, actor, action, ts_hlc, kit_serial, station_code | C | T1-audit |
| KitAttestation | kit_serial, firmware_digest, tpm_quote, sent_at | C | T1-audit |
| KitCustodyEvent | kit_serial, from_officer, to_officer, seal_uids, ts | C | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/bvr/device-ingest.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /bvr/v1/enrolments | internal gRPC | SVID mTLS | yes (Idempotency-Key per record) | Kit to bvr-svc; signed payload |
| POST /bvr/v1/attestations | internal gRPC | SVID mTLS | yes | Boot attestation from kit |
| POST /bvr/v1/transfers | internal gRPC | SVID mTLS | yes | Voter transfer events |
| GET /bvr/v1/handoff/{token} | 3 accredited | SVID mTLS | n/a | Fetch VPRS pre-filled form |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.bvr.enrolment_captured.v1 | emits | eel.bvr.voter.v1 | operational | yes (Ed25519) | at-least-once idempotent |
| eel.bvr.transfer_initiated.v1 | emits | eel.bvr.voter.v1 | operational | yes | at-least-once idempotent |
| eel.bve.kit_attested.v1 | emits | eel.bve.kit.v1 | operational | yes | at-least-once idempotent |
| eel.bve.kit_health_reported.v1 | emits | eel.bve.kit.v1 | analytics | yes | at-least-once idempotent |
| eel.bve.distress_detected.v1 | emits | eel.bve.kit.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.config_bundle_updated.v1 | consumes | eel.bvr.config.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| BVR back-end (M02) | internal | integrations/bvr-backend/ | yes | circuit breaker; queue locally up to 5 000 records |
| ABIS-lite on-device | internal | crates/biometric-edge-abis-lite/ | yes | dedup skipped if lite cache unavailable; flag for server check |
| Geo-fence service (M18) | P01 | integrations/boundary-geofence/ | yes | circuit breaker; last cached polygon used; alert if stale >1 h |
| Kit PKI (P04) | I07 | integrations/kit-pki/ | yes | fail closed; kit refuses enrolment without valid SVID |
| MDM (O01) | internal | integrations/mdm/ | yes | OTA deferred; current firmware continues |

## ML and AI

- On-device ABIS-lite: fingerprint minutiae matcher against last-N enrolments in station cache. Model is a compact ISO 19794-2 matcher compiled to ARM via Rust FFI. Quality gate: NFIQ2 score, reject below threshold.
- Facial quality gate: CNN-based pose and occlusion checker; runs on-device, no templates stored.

## Dependencies

- Modules: M02 (BVR back-end, ingest), M03 (ABIS, server-side dedup), M18 (geofence polygons)
- Platform: P01 (geofence), P04 (kit PKI/SVID), P06 (HSM countersign on submit)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-resilience-rs, kit-sdk-flutter, eel-biometric-rs

## Feature checklist

1. FR-M01-001: capture all 22 mandated biographic fields including INTERSEX gender option.
2. FR-M01-002: capture 10 fingerprints with FBI PIV IQS sensor at FAP 50/60; ISO 19794-2 minutiae; WSQ at 500 PPI.
3. FR-M01-003: capture both iris images with ISO 19794-6 encoding.
4. FR-M01-004: capture ICAO 9303 / ISO 19794-5 facial image with automated quality gate (pose, lighting, expression).
5. FR-M01-005: capture on-screen signature with stylus.
6. FR-M01-006: scan 2nd-gen ID, Maisha card, or passport with MRZ parse and encrypted-PDF output.
7. FR-M01-007: on-device ABIS-lite dedup against last-N station enrolments.
8. FR-M01-008: client-side validation: names alpha-only, DOB >= 18, MRZ validity, passport expiry >= 6 months.
9. FR-M01-009: offline-first encrypted queue; all records survive power failure and sync on reconnect.
10. FR-M01-010: biometric login for Enrolment Clerk and Supervisor roles.
11. FR-M01-011: voter transfer workflow authenticated by Supervisor biometric.
12. FR-M01-012: configurable per-station capacity (default 700); QR-code station lock.
13. FR-M01-013: enrolment window enforcement per Elections Act s.6A; refuse outside gazetted window.
14. FR-M01-014: per-action audit trail (who, what, timestamp) synced to back-end.
15. FR-M01-015: screen-lock after configurable inactivity; biometric re-auth to resume.
16. FR-M01-016: language toggle English / Kiswahili; extensible to 11 languages.
17. FR-M01-017: tactile-overlay input, audio-guided workflow, high-contrast mode; WCAG 2.1 AA.
18. FR-M01-018: distress finger triggers covert alert while displaying normal auth flow.
19. FR-M01-019: kit-health telemetry (battery, connectivity, storage, temperature) at configurable interval.
20. FR-M01-020: signed OTA firmware update with A/B slots and rollback.
21. Clerk session ends and records are sealed when enrolment window closes.
22. Supervisor-approved transfer generates a signed TransferEvent submitted to back-end.
23. Per-record digital signature binds record to kit serial and clerk identity.
24. Sync progress visible to clerk: pending count, last sync timestamp.
25. Kit refuses operation outside geo-fence polygon; alert sent to back-end.
26. Kit refuses operation if NTP/GNSS time confidence is low for more than configured tolerance.
27. Firmware attestation mismatch at boot quarantines kit centrally within 60 s.
28. Kit certificate revoked centrally propagates in <= 60 s; kit becomes inoperable.
29. Local ABIS-lite cache managed with LRU eviction; cache size configurable per station load.
30. MRZ parse pre-populates biographic fields; clerk confirms before biometric capture.
31. QR confirmation slip displayed/printed after successful enrolment with voter reference.
32. Double-feed detection on document scanner; re-scan prompt on failure.
33. Battery and storage thresholds trigger progressive warnings to clerk.
34. Legacy Android feature-matrix: documents which modalities degrade and how.
35. Per-field i18n for all 22 biographic fields across 11 languages.

## Acceptance criteria

AC-M01-01: end-to-end enrolment completes in <= 3 min under field conditions (field trial).
AC-M01-02: offline for 4 h with 100 records enrolled; all records sync on reconnect with zero loss (bench).
AC-M01-03: fingerprint template NFIQ2 >= 60 for >= 95% of captures on certified sensor (bench).
AC-M01-04: distress finger triggers silent alert to back-end with no visible cue to bystanders (red-team).
AC-M01-05: geo-fence violation blocks enrolment and sends alert within 30 s (field trial).
AC-M01-06: firmware attestation deviation quarantines kit centrally within 60 s (chaos exercise).
AC-M01-07: all 22 biographic fields present; INTERSEX gender selectable; field validation passes (manual).
AC-M01-08: screen-reader traversal completes full enrolment flow on TalkBack (accessibility audit).
AC-M01-09: legacy Android runs with documented feature degradation on lowest-tier IEBC device (bench).
AC-M01-10: boot-to-ready within 90 s; >= 12 enrolments/hr sustained throughput (SLA bench).
AC-M01-11: zero records lost on simulated power failure at each workflow state transition (property test).
AC-M01-12: kit revocation propagates from central in <= 60 s; kit refuses new sessions (chaos exercise).

## Applicable edge cases

**connectivity**
EC-CON-01: offline 7-day queue must survive; ring-buffer with signed-sequenced records prevents loss.
EC-CON-02: 3G SLAs met; CBOR+zstd negotiated for submission payloads.
EC-CON-03: 2G only; kit refuses data path; uses SMS/USSD fallback signing.
EC-CON-05: flapping link; backoff+jitter retry; server dedup by Idempotency-Key.
EC-CON-06: captive portal; pinning fails closed; "network blocked" UX; no token leak.
EC-CON-07: satellite 600 ms+ RTT; transport-tuned timeouts; MTU-safe payloads.
EC-CON-08: DNS poisoned; hardcoded fallback endpoints; DNSSEC.
EC-CON-09: IPsec tunnel drop mid-batch; resume at sequence; idempotent re-send.
EC-CON-10: dual-SIM failover; same signed blob on SIM B.

**time**
EC-TIM-01: kit clock skew >5 min at boot; refuse session until NTS/GNSS re-synced.
EC-TIM-02: NTP/NTS unreachable; last-auth time + monotonic; time_confidence=low.
EC-TIM-04: DST/TZ on diaspora kits; UTC storage, gazetted TZ display.

**identity**
EC-IDN-01: kit cert revoked mid-session; refuse within 60 s; queue quarantined.
EC-IDN-03: SVID rotates during batch; old SVID valid to expiry.
EC-IDN-07: no SMS OTP for accredited clerk; device-bound credential only.
EC-IDN-08: duress finger; covert ECC alert; UI identical to normal auth.

**data**
EC-DAT-01: duplicate submission same key+body; cached response; no re-execution.
EC-DAT-02: same key different body; 422 idempotency-key-reuse.
EC-DAT-09: Unicode names; NFC normalisation; ICU collation; grapheme-safe.

**security**
EC-SEC-02: tamper/chassis seal; kill-on-open; quarantine stream; ECC alert.
EC-SEC-05: injection on every input; parameterised only; Semgrep clean.
EC-SEC-11: timing side-channel on biometric compare; constant-time.
EC-SEC-14: stolen kit honey mode; sandbox served; attacker biometric captured.
EC-SEC-16: secrets in logs; telemetry refuses Tier A/B.
EC-SEC-17: PQ hybrid; TLS 1.3 only; never 1.2.

**human**
EC-HUM-01: worn fingerprints; multi-finger retry; s.44A-compliant fallback.
EC-HUM-02: amputee; face/ID path; biometric_exception; no denial.
EC-HUM-03: gloves/wet/dust; liveness reject + prompt.
EC-HUM-04: low-light/direct sun; auto-exposure; torch; quality gate.
EC-HUM-05: assisted voting; assistant recorded; consent; audio.
EC-HUM-06: screen readers; full task completion AA.
EC-HUM-08: 11 languages; no truncation; ICU plurals; font fallback.
EC-HUM-09: colour-blind; symbol+text+pattern.
EC-HUM-11: Braille/tactile overlay; >= 48 dp calibrated; audio confirm.

**kit**
EC-KIT-01: GNSS jam/spoof; multi-constellation + IMU; refuse outside fence.
EC-KIT-02: legacy Android parity; software attestation tier; parity matrix.
EC-KIT-03: OTA interrupted; A/B slots; rollback; signed only.
EC-KIT-04: mode switch not propagated; signed bundle acked; unacked listed in ECC.

**operations**
EC-OPS-01: battery death mid-session; journal-before-act; exact resume; no double token.
EC-OPS-02: kit swap <= 10 min; inherit roll + watermark; old revoked.
EC-OPS-03: seal broken in transit; custody event; quarantine; alert.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | biographic field validators, MRZ parser, WSQ encoder, NFIQ2 gate, window-fence FSM, distress-finger matcher | nextest (Rust), Flutter test | M01-UNIT- |
| property | queue never drops on arbitrary power-cut sequence; ABIS-lite never returns false accept above threshold | proptest | M01-PROP- |
| contract | bvr-backend ingest endpoint; attestation endpoint; VPRS handoff | Pact (consumer M01, provider M02/M04) | M01-PACT- |
| integration | offline queue drain on reconnect; geo-fence enforcement with boundary-svc double | nextest + testcontainers | M01-INT- |
| e2e | full enrolment flow on emulator and real IEBC kit; legacy Android matrix | patrol (Flutter) + HIL bench | M01-E2E- |
| fuzz | MRZ parser; biographic field inputs; sync payload deserialiser | cargo-fuzz | M01-FUZZ- |
| load | 12 enrolments/hr sustained 10 h; sync throughput at 1 000 records | k6 behind toxiproxy | M01-LOAD- |
| chaos | power cut at every FSM transition; cert revocation; OTA interrupt | netem + kill -9 | M01-CHAOS- |
| security | OWASP Mobile Top 10; binary integrity; Semgrep Tier A/B log ban; cert-pin bypass attempt | OWASP scan + Semgrep CI | M01-SEC- |
| a11y | WCAG 2.1 AA; TalkBack full flow; 48 dp hit targets; audio-guided | axe-core + manual TalkBack | M01-A11Y- |
| i18n | 11 locales; 150% pseudo-locale; no truncation; ICU plural | Flutter intl test | M01-I18N- |

Tags follow pattern: `#[trace("FR-M01-001","AC-M01-01","EC-KIT-01")]`

## Runbooks and dashboards

- docs/runbooks/biometric-edge-svc-degradation.md (create): offline-queue overflow; sensor failure; cert revocation procedure.
- docs/runbooks/bvr-device-legacy-android.md (create): feature-matrix degradation steps; parity-test evidence.
- platform/observability/biometric-edge-svc/grafana-kit-health.json: battery, connectivity, queue depth, enrolment rate, NFIQ2 distribution per station.
- platform/observability/biometric-edge-svc/slo-alerts.yaml: enrolment throughput SLO burn rate; queue depth breach.

## Open questions

- Sensor SDK coupling: FBI IQS certification timeline for the shortlisted sensor vendor is not confirmed; affects FR-M01-002.
- Legacy Android feature matrix: exact list of degraded modalities on lowest-tier IEBC devices needs sign-off from IEBC ICT before G3.
- ISO/FBI cert for ABIS-lite on-device matcher is the same risk as M03; both should be resolved together.
