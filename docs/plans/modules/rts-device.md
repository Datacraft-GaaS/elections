# M08 rts-device

| field | value |
|---|---|
| M-number | M08 |
| Home service | rts-svc (ADR-0020) |
| Owning work package | WP-3.2 (docs/plans/04-wave-3-critical-path.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | yes (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M08-rts-device.md |

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

1. FR-M08-001: scan Form 34A with auto-alignment, double-feed detection, encrypted PDF output.
2. FR-M08-002: support all elective-position forms (34A, 35A, 36A, 37A, 38A, MCA, referenda); automatic naming by station code + election + position.
3. FR-M08-003: image quality validation; reject and prompt re-scan if OCR-hostile.
4. FR-M08-004: embed per-station QR code + digital signature at scan time; binds station + kit + form + election.
5. FR-M08-005: PO biometric signature required to authorise transmission; party agent countersignature optional.
6. FR-M08-006: transmit to RTS back-end over telco VPN with mTLS; kit-certificate auth.
7. FR-M08-007: retry with exponential backoff + jitter; local signed queue survives reboot.
8. FR-M08-008: SMS/USSD signed fallback: compressed signed Form 34A summary; back-end reconstructs.
9. FR-M08-009: regular event reports during and after transmission (opening, closing, turnout, incident, attempt, result).
10. FR-M08-010: local audit log of every submission attempt with signature + timestamp + GPS fix.
11. FR-M08-011: post-transmission local retention of signed form image until end-of-election-cycle.
12. FR-M08-012: confirmation slip with form ID, hash, transmission status, transparency-log index.
13. FR-M08-013: party agent QR verification on own phone confirms transmitted form matches physical form.
14. FR-M08-014: LoRa mesh third-tier fallback for enabled stations.
15. Scan + sign + transmit <= 90 s per form under normal conditions.
16. SMS fallback end-to-end <= 2 min; compressed signed submission.
17. Bandwidth budget <= 200 KB compressed per form on average.
18. Queue survives 7-day offline accumulation without data loss.
19. Multi-carrier failover in <= 30 s.
20. Participation token issued for every authenticated voter before poll closes.
21. Party agent countersignature can be added after PO-only transmission within configured window.
22. Each form transmission gets a unique Idempotency-Key preventing server-side duplicates.
23. Kit refuses to scan if GPS fix unavailable and geo-fence unknown.
24. All 6 simultaneous position pipelines run independently; failure in one does not block another.

## Acceptance criteria

AC-M08-01: scan + sign + transmit <= 90 s per form (bench).
AC-M08-02: SMS fallback delivers signed submission end-to-end (field trial).
AC-M08-03: party agent QR verification confirms form match (UAT).
AC-M08-04: local queue survives kit reboot; all records retained (chaos exercise).
AC-M08-05: multi-carrier failover in <= 30 s (live carrier flap test).
AC-M08-06: confirmation slip includes transparency-log index (manual test).
AC-M08-07: 7-day offline queue accumulates without loss; drains on reconnect (bench).
AC-M08-08: participation tokens issued for 100% of authenticated voters at test station (integration).

## Applicable edge cases

**connectivity**
EC-CON-01: 7-day offline queue; ring-buffer signed-sequenced; gap audit on sync.
EC-CON-02: 3G SLAs; CBOR+zstd; 200 KB form budget.
EC-CON-03: 2G only; SMS fallback; compressed signed submission.
EC-CON-05: flapping link; backoff+jitter; dedup by Idempotency-Key.
EC-CON-07: satellite 600 ms+ RTT; transport timeouts tuned; MTU-safe.
EC-CON-09: IPsec tunnel drop mid-batch; resume at sequence; idempotent re-send.
EC-CON-10: dual-SIM failover; same signed blob on SIM B; one DB row.

**time**
EC-TIM-01: kit clock skew >5 min; refuse session until re-synced.
EC-TIM-02: NTP/NTS unreachable all day; time_confidence=low; tolerance window.
EC-TIM-07: s.14A close time moves; HSM-signed extension reopens fence.

**identity**
EC-IDN-01: kit cert revoked mid-session; halt; PO uses paper backup.
EC-IDN-03: SVID rotates during batch; old SVID valid to expiry; producer_key_id per event.
EC-IDN-08: duress finger on PO auth; covert alert; normal UI.

**data**
EC-DAT-01: duplicate submission same key+body; cached response; no re-execution.
EC-DAT-02: same key different body; 422 idempotency-key-reuse.
EC-DAT-04: replayed signed event; reject on counter.
EC-DAT-11: votes > registered voters in scanned form; block with reason; RO override -> DisputeCase.
EC-DAT-16: legacy kit pre-CloudEvents payload; shim upgrades envelope; signature preserved.

**security**
EC-SEC-01: signature mismatch on submission; back-end rejects; kit re-signs and retries.
EC-SEC-02: chassis tamper; kill-on-open; quarantine.
EC-SEC-11: constant-time biometric compare on PO auth.
EC-SEC-14: stolen kit honey mode.
EC-SEC-17: PQ hybrid; TLS 1.3 only.

**kit**
EC-KIT-01: GNSS jam; multi-constellation + IMU.
EC-KIT-02: legacy Android parity; software attestation tier.
EC-KIT-03: OTA interrupted; A/B slots; rollback.
EC-KIT-04: mode switch not propagated; signed bundle acked.

**operations**
EC-OPS-01: battery death mid-session; journal-before-act; exact resume; no double token.
EC-OPS-02: kit swap <= 10 min; streams linked; old revoked.

**legal**
EC-LEG-01: candidate withdraws post-print; status frozen; tallied + annotated.
EC-LEG-03: s.44A invoked; dual-officer S44AEvent; manual IDs logged.

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
- LoRa mesh option: FR-M08-014 is marked optional; decision on which stations are "high-risk" and require LoRa needs IEBC Security sign-off before kit firmware freeze.
