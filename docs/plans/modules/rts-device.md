# M08 rts-device

| field | value |
|---|---|
| M-number | M08 |
| Home service | rts-svc (ADR-0020) |
| Owning work package | WP-3.2 (docs/plans/04-wave-3-critical-path.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | yes (ADR-0013) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M08-rts-device.md |

## Purpose

At close of poll the Presiding Officer scans Form 34A (and position equivalents), the kit binds the scan to station, election, kit, and signer via HSM-backed signature, then transmits over telco VPN with retry and an SMS/USSD fallback so results reach the tally centre even when data connectivity fails entirely.

## Surfaces and languages

| surface | technology | path under services/rts-svc/ |
|---|---|---|
| Kit RTS app | Flutter on packages/kit-sdk-flutter | apps/rts-kit/ |
| Legacy Android shim | Flutter + feature-matrix degradation | apps/rts-kit/ |
| Local event log daemon | Rust | crates/rts-device-core/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| FormScan | scan_id, station_code, position_code, pdf_ref, image_hash, quality_score, signed_at, kit_serial | C | MinIO (on-device encrypted) |
| TransmissionAttempt | attempt_id, scan_id, channel, status, ts, seq | C | T1-audit |
| ParticipationToken | token_sha256, station_code, issued_at, kit_serial | D | T1-audit |
| KitAuditEvent | event_id, actor, action, ts_hlc, gps_fix, kit_sig, seq | C | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/rts/device-ingest.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /rts/v1/forms | internal gRPC | SVID mTLS | yes (Idempotency-Key per scan) | Primary data transmission; mTLS kit auth |
| POST /rts/v1/forms/sms-fallback | 5 stream | SVID mTLS | yes | SMS/USSD compressed fallback intake |
| POST /rts/v1/tokens | internal gRPC | SVID mTLS | yes | Participation token batch submission |
| POST /rts/v1/events | internal gRPC | SVID mTLS | yes | Audit event stream (open, close, turnout, incident) |
| GET /rts/v1/acks/{scan_id} | internal gRPC | SVID mTLS | n/a | Confirmation slip; transparency-log index |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.rts.form_captured.v1 | emits | eel.rts.form.v1 | event-source | yes (Ed25519 + HSM) | exactly-once |
| eel.rts.transmission_attempted.v1 | emits | eel.rts.form.v1 | operational | yes | at-least-once idempotent |
| eel.rts.transmission_succeeded.v1 | emits | eel.rts.form.v1 | operational | yes | at-least-once idempotent |
| eel.rts.participation_token_issued.v1 | emits | eel.rts.token.v1 | event-source | yes | exactly-once |
| eel.rts.poll_opened.v1 | emits | eel.rts.session.v1 | event-source | yes | exactly-once |
| eel.rts.poll_closed.v1 | emits | eel.rts.session.v1 | event-source | yes | exactly-once |
| eel.rts.config_bundle_updated.v1 | consumes | eel.rts.config.v1 | operational | yes | at-least-once idempotent |
| eel.bve.kit_revoked.v1 | consumes | eel.bve.kit.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| RTS back-end (M10) | internal | integrations/rts-backend/ | yes | exponential backoff; SMS/USSD fallback after N fails |
| Telco VPN | I03 | integrations/telco-vpn/ | yes (netem sim) | multi-carrier failover; fallback to SMS/USSD |
| SMS/USSD gateway | I08 | integrations/sms-gw/ | yes | compressed signed fallback; back-end reconstructs |
| Kit PKI (P04) | I07 | integrations/kit-pki/ | yes (SoftHSM) | fail closed; no transmission without valid SVID |
| LoRa mesh (optional) | I14 | integrations/lora-mesh/ | yes (simulated) | third-tier fallback for high-risk stations |

## ML and AI

none (ICR/OCR runs server-side in M10)

## Dependencies

- Modules: M10 (RTS back-end, ingest target), M14 (participation token recipient)
- Platform: P04 (kit PKI/SVID), P05 (audit/Merkle), P06 (HSM)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-resilience-rs, kit-sdk-flutter

## Feature checklist

Scanning and form capture:
1. FR-M08-001: scan Form 34A with auto-alignment, deskew, double-feed detection, and encrypted-PDF output; SSIM >= 0.95. (spec FR-M08-001)
2. FR-M08-002: support all elective-position forms: 34A (President), 35A (Governor), 36A (Senator), 37A (Woman Rep), 38A (MP), MCA, and referenda equivalents; automatic naming by station code, election, and position. (spec FR-M08-002)
3. FR-M08-003: image quality validation: reject and prompt re-scan on OCR-hostile images (skew, blur, missing corners, resolution below spec). (spec FR-M08-003)
4. FR-M08-004: embed per-station QR code at scan time binding station code, kit serial, form type, and election event. (spec FR-M08-004)
5. FR-M08-005: Ed25519 digital signature applied at scan via HSM-backed kit key; covers image hash, station code, timestamp, and kit serial. (spec FR-M08-004)
6. FR-M08-006: all 6 simultaneous position pipelines run independently; failure in one does not block others.
7. FR-M08-007: form bandwidth budget <= 200 KB compressed per form on average; CBOR+zstd negotiated on all channels.
8. FR-M08-008: post-transmission local retention of signed form image until end-of-election-cycle; encrypted at rest. (spec FR-M08-011)

Authentication and signing:
9. FR-M08-009: Presiding Officer biometric signature required to authorise transmission; reject if NFIQ2 below threshold. (spec FR-M08-005)
10. FR-M08-010: party agent countersignature via biometric or issued PIN; optional at initial transmission; addable within configurable window. (spec FR-M08-005)
11. FR-M08-011: duress finger detection: covert ECC alert to M16 while displaying normal auth flow to bystanders.
12. FR-M08-012: participation token issued for every authenticated voter before poll close; batch signed and submitted to M10 and M14.
13. FR-M08-013: participation token count reconciled with total voter-auth count; discrepancy alert sent to ECC.
14. FR-M08-014: party agent countersignature window configurable (default 30 min post-PO-sign); auto-closes and logs if missed.

Transmission:
15. FR-M08-015: primary transmission to RTS back-end (M10) over telco VPN with mTLS; kit-certificate authentication. (spec FR-M08-006)
16. FR-M08-016: exponential backoff + jitter retry on network failure; local signed queue survives kit reboot and power failure. (spec FR-M08-007)
17. FR-M08-017: each form transmission carries a unique Idempotency-Key preventing server-side duplicate execution. (spec FR-M08-007)
18. FR-M08-018: SMS/USSD signed fallback: compressed binary-encoded Form 34A summary via SMS short code; back-end reconstructs and verifies. (spec FR-M08-008)
19. FR-M08-019: SMS fallback compressed to <= 5 SMS segments; end-to-end delivery <= 10 min from initiation to back-end ack.
20. FR-M08-020: LoRa mesh third-tier fallback for flagged high-risk stations; constituency uplink node relays; latency <= 15 min accepted. (spec FR-M08-014)
21. FR-M08-021: dual-SIM failover in <= 30 s: same signed payload retransmitted on SIM B; back-end deduplicates by Idempotency-Key. (spec FR-M08-006)
22. FR-M08-022: multi-carrier failover alert logged and telemetered to ECC when failover triggers.

Event reporting:
23. FR-M08-023: poll-opened and poll-closed events signed by kit and submitted as event-sourced records. (spec FR-M08-009)
24. FR-M08-024: regular event reports: poll open, periodic turnout, incident, transmission attempt, and success or failure. (spec FR-M08-009)
25. FR-M08-025: local audit log of every submission attempt: signature, timestamp, GPS fix, channel, and sequence number. (spec FR-M08-010)
26. FR-M08-026: confirmation slip displayed or printed: form ID, image hash, transmission status, transparency-log index, and QR. (spec FR-M08-012)
27. FR-M08-027: transparency-log index returned in M10 acknowledgement; stored locally and displayed on slip. (spec FR-M08-012)

Party agent verification:
28. FR-M08-028: party agent QR verification on own phone: agent scans transmitted form QR, confirms hash matches physical form. (spec FR-M08-013)
29. FR-M08-029: party agent absence recorded; PO may proceed unilaterally with recorded justification; absence audit trail mandatory.
30. FR-M08-030: Elections Act s.39(1C) compliance: electronic transmission logged as statutory evidence with TLA+-proven state machine.

Offline and resilience:
31. FR-M08-031: 7-day signed offline queue without data loss; ring-buffer with gap audit on sync.
32. FR-M08-032: kit refuses scan if GPS fix unavailable and geo-fence polygon unknown; ECC alert sent within 60 s.
33. FR-M08-033: ballot serial chain (M63) reconciliation: ballot count submitted post-transmission; mismatch creates DisputeCase.

Kit management:
34. FR-M08-034: boot-time firmware integrity check via TPM quote; quarantine centrally within 60 s on mismatch.
35. FR-M08-035: kit clock verified at boot: refuse session if skew > 5 min until NTS/GNSS resynced.
36. FR-M08-036: kit certificate revocation propagated in <= 60 s; kit halts; PO uses paper backup.
37. FR-M08-037: battery and storage threshold progressive warnings to Presiding Officer.
38. FR-M08-038: screen-lock after configurable inactivity; biometric re-auth to resume.
39. FR-M08-039: kit health telemetry (battery, connectivity, storage, temperature, GPS accuracy) at configured interval to ECC.
40. FR-M08-040: escalation to ECC within 60 s: all channels exhausted, GPS fix lost, or kit cert revoked.
41. FR-M08-041: signed OTA firmware update with A/B slots; rollback on integrity failure.
42. FR-M08-042: configuration bundle hot-update via eel.rts.config_bundle_updated.v1 without restart.
43. FR-M08-043: legacy Android feature-matrix degradation: documents which modalities degrade and how on lowest-tier device.

Language and accessibility:
44. FR-M08-044: multilingual: English and Kiswahili on all screens; extensible to 11 languages per constituency profile.
45. FR-M08-045: WCAG 2.1 AA: audio-guided workflow, high-contrast mode, >= 48 dp touch targets on all controls.

Security:
46. FR-M08-046: Semgrep Tier A/B log ban: no PII or biometric data in telemetry or error logs.
47. FR-M08-047: PQ hybrid TLS 1.3 only; no TLS 1.2 fallback; cert pinning with fail-closed on mismatch.
48. FR-M08-048: constant-time biometric compare on PO and party agent authentication; no timing side channel.
49. FR-M08-049: stolen-kit honey mode: on revocation, sandbox environment served; attacker biometric captured.
50. FR-M08-050: form scan quality score logged and transmitted with form; used by M10 ICR engine as confidence prior.

## Acceptance criteria

AC-M08-01: scan + sign + transmit <= 90 s per form under LTE conditions (bench).
AC-M08-02: SMS fallback delivers signed submission end-to-end within 10 min (field trial).
AC-M08-03: party agent QR verification confirms transmitted form hash matches physical form (UAT).
AC-M08-04: local signed queue survives kit reboot; all records retained and synced without loss (chaos exercise).
AC-M08-05: dual-SIM failover in <= 30 s; back-end receives exactly one copy (live carrier flap test).
AC-M08-06: confirmation slip includes transparency-log index (manual test).
AC-M08-07: 7-day offline queue accumulates without loss; drains without duplication on reconnect (bench).
AC-M08-08: participation tokens issued for 100% of authenticated voters; no token missing or duplicated (integration test).
AC-M08-09: kit certificate revocation propagated in <= 60 s; kit halts and refuses new sessions (chaos exercise).
AC-M08-10: GPS fix lost; kit refuses scan; alert sent to ECC within 60 s (HIL bench).
AC-M08-11: WCAG 2.1 AA: axe zero violations; audio-guided workflow completes full scan-sign-transmit (accessibility audit).
AC-M08-12: SSIM >= 0.95 on transmitted form image vs physical; quality rejection rate < 5% on certified scanner (bench).
AC-M08-13: 11 locales: no truncated strings on confirmation slip; ICU plural forms correct (i18n test).
AC-M08-14: availability 99.99% over election-day window (SLO burn-rate alert).
AC-M08-15: TLA+ formal proof of no double-counting and no lost write submitted as regulatory evidence.

## Applicable edge cases

connectivity:
EC-CON-01: 7-day offline queue; ring-buffer signed-sequenced; gap audit on reconnect sync.
EC-CON-02: 3G SLAs met; CBOR+zstd negotiated; 200 KB form budget enforced.
EC-CON-03: 2G only; data path refused; SMS/USSD fallback activated with compressed signed submission.
EC-CON-05: flapping link; exponential backoff + jitter; dedup by Idempotency-Key.
EC-CON-06: captive portal; cert pinning fails closed; "network blocked" UX; no token leaked.
EC-CON-07: satellite 600 ms+ RTT; transport timeouts tuned; MTU-safe payloads.
EC-CON-08: DNS poisoned; hardcoded fallback endpoints; DNSSEC enforced.
EC-CON-09: IPsec tunnel drop mid-batch; resume at sequence; idempotent re-send.
EC-CON-10: dual-SIM failover; same signed blob on SIM B; back-end deduplicates to one row.

time:
EC-TIM-01: kit clock skew > 5 min at boot; refuse session until NTS/GNSS resynced.
EC-TIM-02: NTP/NTS unreachable all day; time_confidence=low; tolerance window applied.
EC-TIM-04: DST/TZ: UTC storage; gazetted EAT display; poll close computed in UTC.
EC-TIM-06: event time < causation time in audit log; accept; audit skew anomaly recorded.
EC-TIM-07: s.14A close time extended; HSM-signed extension reopens fence; forged extension rejected.
EC-TIM-09: register freeze race T-60; OPA hard-close at tx time; all-or-nothing token batch.

identity:
EC-IDN-01: kit cert revoked mid-session; halt within 60 s; PO uses paper backup.
EC-IDN-02: JWT expires mid-request on back-end path; in-flight completes; next 401 + silent refresh.
EC-IDN-03: SVID rotates during batch; old SVID valid to expiry; producer_key_id per event.
EC-IDN-08: duress finger on PO auth; covert ECC alert; UI identical to normal auth flow.
EC-IDN-10: SpiceDB down on back-end; fail closed on writes; class-C reads only.

data:
EC-DAT-01: duplicate form submission same key+body; cached response; no re-execution.
EC-DAT-02: same key different body; 422 idempotency-key-reuse.
EC-DAT-04: replayed signed event; reject on monotonic counter.
EC-DAT-11: votes > registered voters in scanned form; block with reason; RO override creates DisputeCase.
EC-DAT-16: legacy kit pre-CloudEvents payload; shim upgrades envelope; signature preserved.

security:
EC-SEC-01: signature mismatch on submission; back-end rejects; kit re-signs and retries.
EC-SEC-02: chassis tamper or seal break; kill-on-open; quarantine stream; ECC alert.
EC-SEC-05: injection on all user inputs; parameterised only; Semgrep clean.
EC-SEC-11: constant-time biometric compare on PO and agent auth; no timing side channel.
EC-SEC-14: stolen kit honey mode; sandbox served; attacker biometric captured.
EC-SEC-16: secrets in logs refused; telemetry refuses Tier A/B fields.
EC-SEC-17: PQ hybrid; TLS 1.3 only; never TLS 1.2.

kit:
EC-KIT-01: GNSS jam or spoof; multi-constellation + IMU; refuse outside geo-fence.
EC-KIT-02: legacy Android parity; software attestation tier; policy-marked; parity matrix tested.
EC-KIT-03: OTA interrupted; A/B slots; rollback; signed update only.
EC-KIT-04: mode switch not propagated; signed bundle acked; unacked listed in ECC.

operations:
EC-OPS-01: battery death mid-session; journal-before-act; exact resume; no double token.
EC-OPS-02: kit swap <= 10 min; streams linked; old kit revoked; watermark inherited.
EC-OPS-03: seal broken in transit; custody event; quarantine; alert to ECC.
EC-OPS-06: flood or curfew relocation; HSM-signed geo-fence polygon update; portal notice.
EC-OPS-07: staff no-show or swap; credential exactly-once; old credential revoked.
EC-OPS-08: disk full on kit; bounded queue; 507 surfaced to PO; no corruption.

scale:
EC-SCL-04: consumer lag > 60 s on form topic at back-end; backpressure applied; no OOM.
## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | form quality gate, signature binding, retry FSM, SMS compression, participation token generation | nextest + Flutter test | M08-UNIT- |
| property | queue never drops under arbitrary power-cut; idempotency on all channels; signed counter monotonic | proptest | M08-PROP- |
| contract | rts-backend form ingest (provider M10); SMS fallback receiver (provider M10) | Pact | M08-PACT- |
| integration | offline drain on reconnect; SMS fallback end-to-end; LoRa mesh double | nextest + testcontainers | M08-INT- |
| e2e | full scan -> sign -> transmit -> confirmation slip on real kit | patrol + HIL bench | M08-E2E- |
| fuzz | form payload; QR scan; SMS decompression; GPS coordinate | cargo-fuzz | M08-FUZZ- |
| formal | queue append-only; participation token uniqueness; no double-token per voter | kani | M08-FORMAL- |
| load | 90 s cycle; SMS fallback under load; 200 KB budget | k6 + carrier sim | M08-LOAD- |
| chaos | power cut at every step; cert revocation; carrier flap; OTA interrupt | netem + kill -9 | M08-CHAOS- |
| security | OWASP Mobile Top 10; binary integrity; Semgrep; SMS injection | OWASP + Semgrep | M08-SEC- |
| i18n | 11 locales; form labels and confirmation slip | Flutter intl | M08-I18N- |

## Runbooks and dashboards

- docs/runbooks/rts-svc-degradation.md (create): SMS fallback procedure; kit revocation mid-transmission; LoRa activation.
- docs/runbooks/rts-device-battery.md (create): power-cut recovery steps; confirmation slip re-print.
- platform/observability/rts-svc/grafana-kit-tx.json: transmission success rate, SMS fallback rate, queue depth, form quality pass rate, participation token count.
- platform/observability/rts-svc/slo-alerts.yaml: 99.99% burn rate; scan-sign-transmit > 90 s; SMS fallback SLA breach.

## Open questions

- SMS compression of signed forms: the compressed signed summary format needs a published spec agreed with telco partners before G3; current design uses a custom binary encoding.
- LoRa mesh option: FR-M08-014 is marked optional; decision on which stations are "high-risk" and require LoRa needs the Commission Security sign-off before kit firmware freeze.
