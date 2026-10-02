# M01 bvr-device

| field | value |
|---|---|
| M-number | M01 |
| Home service | biometric-edge-svc (ADR-0020) |
| Owning work package | WP-3.8 (docs/plans/04-wave-3-critical-path.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M01-bvr-device.md |

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

Biographic capture
FR-M01-001: capture all 22 mandated biographic fields: name(s), DOB, gender (MALE, FEMALE, INTERSEX), ID type, ID number, ID expiry, mobile, email, education level, physical-challenge type, assistance-required flag, county, constituency, ward, and polling centre.
FR-M01-002: capture 10 fingerprints with FBI PIV IQS or STQC-certified sensor at FAP 50/60; extract ISO/IEC 19794-2 minutiae; encode with FBI-certified WSQ at 500 PPI and JPEG-2000 (ISO/IEC 15444-1:2019) at <= 15:1 average compression.
FR-M01-003: capture both iris images with ISO/IEC 19794-6 encoding; liveness detection rejects artificial iris before template extraction.
FR-M01-004: capture ICAO 9303 / ISO 19794-5 facial image with automated quality gate (pose, lighting, expression); auto-exposure and torch activation on poor illumination.
FR-M01-005: capture on-screen signature with stylus; stylus vector hash stored; raw vectors discarded after hashing.
FR-M01-006: scan 2nd-gen national ID, Maisha card, or passport; auto-alignment; double-feed detection; MRZ parse; encrypted-PDF output with hash reference stored in MinIO.
FR-M01-007: on-device ABIS-lite dedup against last-N station enrolments; LRU eviction manages cache size; cache-miss flagged for server-side check.
FR-M01-008: client-side validation: names alpha-only and NFC-normalised; DOB >= 18 at registration date; MRZ check-digit validity; passport expiry >= 6 months from registration date.

Offline queue and sync
FR-M01-009: offline-first: full enrolment flow works with no network; records stored in encrypted SQLite WAL queue with per-record Ed25519 digital signature; submitted on reconnect.
FR-M01-010: offline queue ring-buffer capacity covers >= 7 days of peak station throughput without loss or overflow; signed-sequenced records allow gap audit.
FR-M01-011: each enrolment record signature binds voter biographic hash, kit serial, clerk biometric ID, and HLC timestamp.
FR-M01-012: idempotent re-send by eel_idempotency_key on reconnect; server dedup prevents duplicate voter records.
FR-M01-013: sync progress visible to clerk: pending record count, last successful sync timestamp, last error code displayed in status bar.

Authentication and access
FR-M01-014: biometric fingerprint login for Enrolment Clerk and Supervisor roles; PIN fallback rate-limited; no downgrade to weaker auth policy.
FR-M01-015: screen-lock after configurable inactivity period; biometric re-authentication required to resume; partial captures discarded on timeout.
FR-M01-016: role-based access enforced per SVID; SVID rotation during session handled without session drop; new SVID accepted on next API call.
FR-M01-017: distress finger (designated at PO briefing) triggers covert eel.bve.distress_detected.v1 event to back-end while UI remains identical to normal auth.
FR-M01-018: revoked-kit login enters honeypot mode: sandbox environment served; attacker biometric silently captured for forensics.

Station and window control
FR-M01-019: enrolment window enforced per Elections Act s.6A; kit refuses new enrolments outside gazetted window; HSM-signed extension bundle reopens window; forged extension rejected.
FR-M01-020: configurable per-station capacity (default 700); QR-code station lock prevents cross-station capture; capacity exhaustion notifies clerk.
FR-M01-021: county, constituency, ward, and polling centre selection validated against loaded electoral-area map bundle; unrecognised codes rejected with reason.
FR-M01-022: MRZ parse pre-populates all matching biographic fields; clerk confirms every field before biometric capture begins.
FR-M01-023: QR confirmation slip displayed and printable after successful enrolment with voter registration reference and station code.
FR-M01-024: clerk session sealed and all pending records signed when enrolment window closes; no new enrolment session until next window.

Voter transfer
FR-M01-025: voter transfer workflow authenticated by Supervisor biometric and originating voter biometric; produces signed eel.bvr.transfer_initiated.v1 event submitted to back-end.
FR-M01-026: KitCustodyEvent produced on kit hand-off between officers; seal UIDs recorded; event submitted to back-end immediately.

Audit and attestation
FR-M01-027: per-action audit trail (who, what, timestamp, kit serial) appended to local SQLite audit table; synced to back-end T1-audit on each batch.
FR-M01-028: KitAttestation (TPM quote + firmware digest) sent as eel.bve.kit_attested.v1 on every boot; kit quarantined centrally within 60 s on mismatch.
FR-M01-029: kit certificate revocation propagated from central SpIFFE/SPIRE within 60 s; kit becomes inoperable; local queue quarantined; custodian alerted.

Firmware and OTA
FR-M01-030: signed OTA firmware and app updates via A/B partitioned slots; automatic rollback on boot failure; unsigned or unverified images refused at install.
FR-M01-031: OTA interrupted at any phase triggers A/B rollback; current running firmware restored; update retried on next MDM push.

Geo-fence and time
FR-M01-032: kit refuses enrolment outside geo-fence polygon; multi-constellation GNSS (GPS, GLONASS, Galileo) with IMU delta for spoof detection; eel_geo_violation alert sent within 30 s.
FR-M01-033: kit blocks operation when NTP/GNSS time confidence is low beyond configured tolerance; eel_time_confidence=low stamped on every affected record.
FR-M01-034: UTC used for all storage and computations; gazetted TZ displayed to clerk; DST transitions handled transparently on diaspora kits.

Health telemetry
FR-M01-035: kit-health telemetry (battery, connectivity, storage, temperature) emitted as eel.bve.kit_health_reported.v1 at configurable interval; escalated to war-room on sensor failure.
FR-M01-036: battery and storage threshold warnings displayed progressively (warning at 20%, critical at 10%); back-end pull-priority triggered on near-full storage.

Biometric quality and exception paths
FR-M01-037: NFIQ2 quality gate: fingerprint rejected below configurable threshold; re-capture prompted up to three attempts before exception path is offered.
FR-M01-038: iris liveness detection on every capture; liveness confidence below threshold triggers reject and re-prompt.
FR-M01-039: multi-finger retry with adaptive threshold for worn, faded, or gloved fingerprints; s.44A-compliant fallback to face-and-ID path documented.
FR-M01-040: biometric exception path (amputee, injury, s.44A): face-and-ID enrolment enabled; eel_biometric_exception flag set; no denial of registration.
FR-M01-041: assisted voting path: assistant identity and biometric recorded; voter consent captured; audio guidance active; assistant identity excluded from voter secrecy audit fields.
FR-M01-042: NFIQ2 distribution per station logged in health telemetry payload; outlier stations alerted via Grafana burn-rate rule.

Accessibility and localisation
FR-M01-043: WCAG 2.1 AA throughout: tactile-overlay-compatible input; audio-guided workflow on every step; high-contrast mode switchable at any point without restart.
FR-M01-044: screen-reader full task completion on TalkBack (Android) and NVDA (Windows); every interactive element has accessible label and role.
FR-M01-045: all interactive controls meet >= 48 dp hit-target size; Braille/tactile overlay calibrated; audio confirmation on each action.
FR-M01-046: language toggle between at least English and Kiswahili; architecture extensible to 11 languages without code change; per-field i18n for all 22 biographic fields.
FR-M01-047: all 11 languages render without truncation at 150% pseudo-locale on every field; ICU plural rules applied; font fallback stack defined per locale.
FR-M01-048: colour-blind support: symbol, text label, and pattern accompany every colour-coded state; ΔE contrast validated per registry.
FR-M01-049: Kenyan Sign Language video available for every clerk-facing explainer flow; captions present for all audio content.

Security and crypto
FR-M01-050: all biometric templates and PII encrypted at rest with per-field keys managed via eel-crypto-rs; no plaintext template observable outside enclave or encrypted store.
FR-M01-051: TLS 1.3 minimum for all outbound connections; PQ-hybrid negotiated where peer supports; TLS 1.2 refused.
FR-M01-052: all log output scanned against eel_pii_tier_a and eel_pii_tier_b regex patterns before write; telemetry pipeline refuses classified fields at the sink.

Legacy and interoperability
FR-M01-053: legacy Android feature-matrix degradation layer documents which modalities degrade on each device tier; policy-marked capabilities enforced at runtime.
FR-M01-054: legacy kit pre-CloudEvents payload shim upgrades envelope to eel.<ctx>.<event>.v1 format; original signature preserved; Pact golden corpus verified.

## Acceptance criteria

AC-M01-01: end-to-end enrolment completes in <= 3 min under field conditions on certified sensor hardware, measured across 20 physical kits in staging environment (field trial).
AC-M01-02: offline for 4 h with 100 records enrolled; all records sync on reconnect with zero loss and zero duplication confirmed by proptest suite (bench).
AC-M01-03: fingerprint templates NFIQ2 >= 60 for >= 95% of captures on FBI PIV IQS certified sensor across 1 000 captures (bench).
AC-M01-04: distress finger triggers silent eel.bve.distress_detected.v1 event to back-end with no visible change in UI or display to bystanders (red-team exercise).
AC-M01-05: geo-fence violation blocks enrolment immediately and sends alert to back-end within 30 s (field trial with simulated boundary crossing).
AC-M01-06: firmware attestation mismatch quarantines kit centrally within 60 s; kit refuses all new sessions after quarantine (chaos exercise).
AC-M01-07: all 22 biographic fields present and validated; INTERSEX gender selectable; MRZ pre-populate correct for 2nd-gen ID, Maisha card, and passport (manual test).
AC-M01-08: screen-reader traversal completes full enrolment flow on TalkBack; WCAG 2.1 AA audit passes with zero critical violations (accessibility audit by independent WCAG specialist).
AC-M01-09: legacy Android feature-matrix executes with documented degradation on lowest-tier the Commission device; parity matrix signed off by the Commission ICT (bench on real the Commission device).
AC-M01-10: kit boot-to-ready within 90 s; >= 12 enrolments/hr sustained throughput over 10 h continuous operation (SLA bench).
AC-M01-11: zero records lost on simulated power failure at every FSM transition; journal-before-act verified by property test with arbitrary cut-point corpus (proptest).
AC-M01-12: kit certificate revocation propagated from central; kit refuses new sessions within 60 s of revocation event (chaos exercise).
AC-M01-13: 11-language UI renders without truncation at 150% pseudo-locale across all 22 biographic fields and all workflow screens (i18n regression test).
AC-M01-14: biometric exception path (amputee, worn fingerprint, s.44A invocation) completes enrolment without denial; biometric_exception flag present in record (UAT with PWD participants).
AC-M01-15: offline queue survives 7-day accumulation at peak station throughput without ring-buffer overflow or record drop (property test).
AC-M01-16: TLS 1.3 minimum enforced on all outbound connections; TLS 1.2 actively rejected on probe (testssl.sh in CI).

## Applicable edge cases

connectivity
EC-CON-01: 7-day offline accumulation; ring-buffer with signed-sequenced records prevents loss; gap audit run on reconnect.
EC-CON-02: 3G (500 kbps, 300 ms RTT); kit meets enrolment submission SLAs; CBOR+zstd negotiated for payloads.
EC-CON-03: 2G only; kit refuses data path; switches to SMS/USSD signed fallback; clerk shown USSD path prompt.
EC-CON-05: flapping link; backoff+jitter retry with budget; server dedup by eel_idempotency_key; zero duplicates.
EC-CON-06: captive portal intercepts; certificate pinning fails closed; "network blocked" UX shown; no auth tokens leaked.
EC-CON-07: satellite 600 ms+ RTT; transport-tuned timeouts applied; MTU-safe payloads avoid fragmentation.
EC-CON-08: DNS poisoned or down; DNSSEC enforced; hardcoded fallback endpoints used; circuit opens on repeated failure.
EC-CON-09: IPsec tunnel drops mid-batch; resume from last committed sequence number; idempotent re-send by key.
EC-CON-10: dual-SIM failover mid-send; same signed blob submitted on SIM B; exactly one record committed.

time
EC-TIM-01: kit clock skew > 5 min at boot; session refused until NTS or GNSS re-sync confirmed.
EC-TIM-02: NTP/NTS unreachable all day; last-auth time + monotonic clock used; eel_time_confidence=low stamped on every record.
EC-TIM-04: DST or TZ change on diaspora kit; UTC stored; gazetted TZ displayed; window close computed in UTC.
EC-TIM-07: s.14A close time moves 17:00 to 19:00; HSM-signed extension bundle reopens fence; forged extension rejected.

identity
EC-IDN-01: kit cert revoked mid-session; refused within 60 s; queue quarantined; honeypot served if theft confirmed.
EC-IDN-02: JWT expires mid-request; in-flight request completes; next request returns 401 with silent refresh.
EC-IDN-08: duress finger recognised; covert eel.bve.distress_detected.v1 sent; UI display identical to normal auth.
EC-IDN-10: SpiceDB down; fail-closed on write operations; class-C reads from local cache; 503 returned.

data
EC-DAT-01: duplicate submission same eel_idempotency_key and body; cached 200 returned; no re-execution.
EC-DAT-02: same eel_idempotency_key different body; 422 idempotency-key-reuse returned.
EC-DAT-09: Unicode names; NFC normalisation applied; ICU collation used; grapheme-safe splitting for display.
EC-DAT-16: legacy kit pre-CloudEvents payload; shim upgrades envelope to eel.bvr.enrolment_captured.v1; original Ed25519 signature preserved.

security
EC-SEC-02: chassis tamper or seal broken; kill-on-open triggered; eel_tamper quarantine stream event; ECC alert sent; custody event generated.
EC-SEC-05: injection attempt on biographic fields; parameterised storage only; Semgrep Tier A/B passes in CI.
EC-SEC-11: timing side-channel on ABIS-lite biometric compare; constant-time comparison enforced in eel-biometric-rs.
EC-SEC-14: stolen kit in honeypot mode; sandbox environment served; attacker biometric silently captured and forwarded.
EC-SEC-16: secrets or PII emitted to logs or traces; telemetry sink refuses eel_pii_tier_a and eel_pii_tier_b fields; CI regex check enforced.
EC-SEC-17: PQ-hybrid negotiation attempted; TLS 1.3 enforced; TLS 1.2 refused on all outbound connections.

human
EC-HUM-01: worn or faded fingerprints; multi-finger retry with adaptive threshold; s.44A-compliant face-and-ID fallback activated.
EC-HUM-02: amputee or missing digits; face-and-ID enrolment path; eel_biometric_exception flag set in record; no denial.
EC-HUM-03: gloves, wet hands, or dust on platen; liveness reject with re-prompt; enrolment latency SLA maintained.
EC-HUM-04: low-light or direct sun environment; auto-exposure triggered; torch activated; perspective correction applied; quality gate enforced.
EC-HUM-05: assisted voting; assistant identity and biometric recorded; consent captured; audio guidance active; no secrecy leak in audit.
EC-HUM-06: screen readers (TalkBack, NVDA); full task completion with assistive tech; WCAG 2.1 AA semantics verified.
EC-HUM-07: Deaf or KSL users; KSL video present on all explainer and instruction flows; captions on all audio.
EC-HUM-08: 11 languages displayed; no truncation at 150% pseudo-locale; ICU plurals applied; font fallback stack defined.
EC-HUM-09: colour-blind operators; symbol, text label, and pattern accompany every colour-coded state; ΔE registry compliant.
EC-HUM-10: low-literacy clerk on USSD path; numeric steps <= 5; IVR alternative available.
EC-HUM-11: Braille or tactile overlay user; >= 48 dp hit targets calibrated; audio confirmation on every action.
EC-HUM-12: clerk fatigue causes wrong column entry; ICR cross-check flags low confidence; forced re-entry before submission.

kit
EC-KIT-01: GNSS jam or spoof attempt; multi-constellation + IMU delta check; enrolment refused when outside verified polygon.
EC-KIT-02: legacy Android device; software attestation tier applied; policy-marked capabilities enforced; parity matrix maintained.
EC-KIT-03: OTA update interrupted at any phase; A/B slot rollback restores running firmware; only signed images accepted.
EC-KIT-04: mode-switch bundle not acknowledged by kit; unacknowledged kit listed in ECC dashboard; signed bundle re-sent on ack timeout.

operations
EC-OPS-01: battery death mid-session; journal-before-act ensures exact FSM resume; no double token or duplicate record.
EC-OPS-02: kit swap required; replacement inherits roll and watermark within <= 10 min; old kit revoked; event streams linked.
EC-OPS-03: seal broken in transit; KitCustodyEvent generated; kit quarantined; alert sent to ECC.
EC-OPS-04: dual-control officer absent at critical step; time-boxed ECC remote co-sign flow triggered.
EC-OPS-06: flood or curfew forces station relocation; HSM-signed geo-fence polygon update only accepted; portal notice issued.

scale
EC-SCL-04: consumer lag > 60 s on submission sync channel; backpressure applied; kit paged; no OOM on queue buffer.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | biographic field validators, MRZ parser, WSQ encoder, NFIQ2 gate, window-fence FSM, distress-finger matcher | nextest (Rust), Flutter test | M01-UNIT- |
| property | queue never drops on arbitrary power-cut sequence; ABIS-lite never returns false accept above threshold | proptest | M01-PROP- |
| contract | bvr-backend ingest endpoint; attestation endpoint; VPRS handoff | Pact (consumer M01, provider M02/M04) | M01-PACT- |
| integration | offline queue drain on reconnect; geo-fence enforcement with boundary-svc double | nextest + testcontainers | M01-INT- |
| e2e | full enrolment flow on emulator and real the Commission kit; legacy Android matrix | patrol (Flutter) + HIL bench | M01-E2E- |
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
- Legacy Android feature matrix: exact list of degraded modalities on lowest-tier the Commission devices needs sign-off from the Commission ICT before G3.
- ISO/FBI cert for ABIS-lite on-device matcher is the same risk as M03; both should be resolved together.
