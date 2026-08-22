# Mobile telemetry and observability

Comprehensive persistent telemetry layer for every kit + mobile + desktop-mobile Flutter app in the platform (13 apps). Tracks location + time + activities + proximal ballot boxes + movements + logins/logouts + reads/writes + battery + locks/unlocks + physical drops (accelerometer). All signals persisted, hash-chained, Merkle-anchored, and queryable by Commission administrators via a natural-language-adjacent query surface. Owned jointly by kit-sdk-flutter (SDK), field-mobile-svc + inventory-svc + biometric-edge-svc (ingest), ai-platform-svc (analytics), and ecc-svc (admin console).

## Contents

1. Goals + SCoK defensibility posture
2. Signal taxonomy (13 signal classes)
3. Data plane (schema)
4. Client-side (kit-sdk-flutter + per-app instrumentation)
5. Ingestion pipeline
6. Storage + retention + Merkle anchoring
7. Analytics + anomaly detection
8. Admin query console (hard-questions-fast surface)
9. Integrity + tamper resistance
10. Privacy + PII posture
11. Honest caveats + external prerequisites
12. Implementation split (4 work packages)

---

## 1. Goals + SCoK defensibility posture

Commission administrators must be able to ask hard operational questions and get prompt + correct + full answers. Examples:

- "Which kits at station 12345 came within 2 metres of a ballot box between 17:00 and 19:00 on election day?"
- "Show me every drop-event > 3g on any kit assigned to Nairobi county in the last 48h."
- "Officer with badge #4432 logged into which devices, when, from which geo, and did they access any voter records outside their assigned station?"
- "How many minutes of accelerometer-detected motion did kit K-8873 record between arrival at station and its first voter identification?"
- "Was every kit that reported a battery below 15% dispatched a replacement or a power bank within the runbook SLA?"

Every answer must be cryptographically anchored, court-defensible, and reproducible under SCoK evidentiary standards (see `docs/features/SC-defensibility.md`).

## 2. Signal taxonomy (13 classes)

| # | Class | Sub-signals | Frequency | Sensor / source |
|---|---|---|---|---|
| 1 | **Location** | GPS lat/lon/accuracy/altitude, cell-tower id + rssi, Wi-Fi BSSID + rssi, computed-position | 15 s (idle) to 1 s (active) | Geolocator + telephony + wifi_scan |
| 2 | **Time** | wall clock, monotonic clock, HLC, NTP offset, last time-fence check | every signal | eel-hlc-rs + platform clock |
| 3 | **Activity** | screen transitions, button presses (redacted), workflow-step markers, keyboard-open, camera-active | per-event | Flutter navigator + gesture recognisers |
| 4 | **Proximal RFID / ballot boxes** | RFID EPC seen (BLE scan or built-in reader), RSSI, seal-tag status, box body id | 5 s scan | RFID service (rfid_reader_service.dart) |
| 5 | **Movements** | accelerometer 100 Hz, gyroscope 100 Hz, magnetometer 50 Hz, motion class (stationary/walking/running/vehicle) | 100 Hz raw, downsampled to 1 Hz for storage | sensors_plus |
| 6 | **Login / logout** | actor SVID + biometric verification result, device state at login, session duration | per-event | eel-auth-rs kit binding |
| 7 | **Reads** | which voter record was queried, which kit-config was fetched, which OTA update was pulled | per-event | Isar-instrumented + HTTP-client hook |
| 8 | **Writes** | which enrolment posted, which identification submitted, which incident filed | per-event | outbox emit hook |
| 9 | **Battery** | percent, health, temperature, charging state, plug-detect (USB / AC), estimated-time-remaining | 30 s | battery_plus + thermal_platform |
| 10 | **Locks / unlocks** | screen-lock state, biometric-unlock event, PIN attempts (redacted), inactivity timeout | per-event | flutter_local_notifications + platform-channel |
| 11 | **Physical drops** | accelerometer spike > 3g threshold, free-fall detection, orientation-change > 90 deg in < 200 ms, chassis-tamper GPIO | per-event | sensors_plus + kit hardware pin |
| 12 | **Network** | connectivity class (offline / 2G / 3G / 4G / 5G / Wi-Fi), effective bandwidth, latency to server, retry-backoff state | per-transition | connectivity_plus |
| 13 | **Attestation** | Play Integrity / App Attest / TPM / TDX status, kit-cert validity, next-cert-refresh | per-heartbeat + on change | eel-abis-attest + kit-attest |

## 3. Data plane (migration `V052__mobile_telemetry.sql`)

New tables in `field-mobile-svc` (owning service; other services consume via read-only views):

```
eel_telemetry_devices        -- one row per registered device (kit or mobile), lifecycle
eel_telemetry_sessions        -- login-through-logout session bounds
eel_telemetry_signals         -- append-only signal stream (partitioned by day)
eel_telemetry_activity_frames -- 1s-downsampled motion + battery envelope
eel_telemetry_geo_dwells       -- computed dwell-time-in-polygon aggregates
eel_telemetry_alerts          -- rule-triggered alerts (drop, battery, geo-fence, unusual)
eel_telemetry_queries         -- admin queries recorded for audit
eel_telemetry_snapshots       -- Merkle-anchored daily snapshot roots
```

- Signals table is monthly partitioned on `occurred_at`; retention 15 y for critical classes (drop, chassis tamper, login/logout, write) + 90 d for high-volume raw sensor streams (accelerometer downsampled to 1 s frames; raw 100 Hz retained 24 h then discarded).
- Per-signal `prev_hash` chain per device — silent rewrite breaks the chain.
- Merkle snapshot per device per day anchored to the P05 transparency log.
- Every signal record carries `trace_id + span_id + correlation_id + causation_id` (W3C traceparent).

## 4. Client-side (kit-sdk-flutter + per-app instrumentation)

New sub-module `packages/kit-sdk-flutter/lib/src/telemetry/`:

- `signal_sink.dart` — abstract signal sink; all classes derive `Signal`.
- `signal_types.dart` — 13 concrete Signal classes matching the taxonomy.
- `sensors_service.dart` — sensors_plus adapter with 100 Hz raw sampling + 1 Hz downsample + drop-event detector (peak > 3g + 200 ms window) + free-fall detector + orientation-change detector.
- `location_service.dart` — Geolocator + telephony + wifi_scan fusion; drops precision below district level when accuracy > 500 m.
- `activity_tracker.dart` — Flutter navigator observer + gesture recognisers + workflow-step marker API.
- `rfid_proximity_service.dart` — BLE scan for RFID tags in range (integrates with cover-custody).
- `session_recorder.dart` — bracket every user session with signed enter/exit.
- `battery_monitor.dart` — battery_plus at 30 s + platform-channel thermal.
- `drop_detector.dart` — accelerometer + gyroscope + free-fall; emits critical Signal.
- `network_watcher.dart` — connectivity_plus + effective bandwidth + latency probe.
- `attestation_pulse.dart` — periodic re-attestation heartbeat.
- `offline_queue.dart` — Isar-backed queue with 7-day capacity + crash-safe journal + sync-on-connect.
- `hash_chain.dart` — per-device prev_hash chain over signals before sync.
- `signer.dart` — HSM-anchored signature per signal via kit-attest.
- `redactor.dart` — PII redaction before egress (never send raw voter IDs; hash-only).
- `sink_dispatcher.dart` — batch + retry + backpressure to the ingest endpoint.

Every one of the 13 Flutter apps calls `TelemetryService.init(appId, personaRole)` on boot; the SDK auto-instruments the rest.

## 5. Ingestion pipeline

Endpoint: `POST /v1/telemetry/ingest` on the new `field-mobile-svc/crates/telemetry-ingest` crate.

```
device app -> HTTPS to field-mobile-svc/telemetry-ingest
   -> verify SVID + kit-cert + HMAC over payload
   -> verify per-device prev_hash chain link
   -> replay-defence via nonce + monotonic counter
   -> classify signal + write to eel_telemetry_signals (partitioned)
   -> emit CloudEvent eel.telemetry.signal.v1 via outbox
   -> if signal class is drop / chassis-tamper / geo-fence-breach / login-anomaly:
        also raise eel.telemetry.alert.v1 + notify ECC (M16)
   -> return ack {last_hash, server_time, next_backoff_ms}
```

- Batch mode: up to 1000 signals per request; server ack includes per-signal accept/reject.
- Rate limits: per-device 100 signals/s hard cap.
- Time-series storage: PG partitioned tables initially; migration to TimescaleDB extension when kernel + license approval lands.
- Alternative for downsampled frames: Parquet in MinIO with daily rollups.

## 6. Storage + retention + Merkle anchoring

Per ADR-0016:

| Class | Retention | Location |
|---|---|---|
| Login/logout | 15 y | eel_telemetry_signals (partitioned) |
| Writes (enrolment, identification, incident) | 15 y | eel_telemetry_signals |
| Reads | 5 y | eel_telemetry_signals |
| Drop / chassis-tamper / geo-fence-breach | 15 y | eel_telemetry_signals + eel_telemetry_alerts |
| Movement raw (100 Hz) | 24 h | ring buffer in MinIO |
| Movement 1 Hz frames | 90 d | eel_telemetry_activity_frames |
| Battery + network | 90 d | eel_telemetry_activity_frames |
| RFID proximity | 15 y | eel_telemetry_signals (audit trail) |
| Attestation | 3 y | eel_telemetry_signals |
| Location fine (< 500 m) | 90 d | eel_telemetry_signals |
| Location coarse (district) | 15 y | eel_telemetry_signals (privacy) |

- Daily Merkle snapshot per device: sha256(sorted per-device signal hashes) → per-device root; sha256(all device roots) → per-day root; anchored to P05 transparency log via `eel_telemetry_snapshots`.
- Per-tag hash chain over `eel_telemetry_signals.prev_hash / this_hash`.
- Append-only triggers on `eel_telemetry_signals` + `eel_telemetry_alerts` blocking UPDATE / DELETE.

## 7. Analytics + anomaly detection

New crate `services/ai-platform-svc/crates/telemetry-analytics`:

- **Drop detection at scale** — every drop signal cross-referenced with the device's assigned custody state; flags physical mistreatment during transit or at station.
- **Geo-fence breach** — kit outside its assigned station polygon during voting hours; feeds ECC dispatch.
- **Session-hijack heuristic** — reuses HARD-14 behavioural-biometrics scoring; adds device-location + battery-state + attestation-status to the feature vector.
- **Battery starvation cluster** — multiple kits in a region reporting low battery → dispatch power banks.
- **RFID absence anomaly** — kit that expected to see a specific ballot box RFID has not seen it for > 15 min during transit.
- **Login-time anomaly** — officer login at unusual time-of-day for that officer + station.
- **Read-heavy anomaly** — an officer session with unusually high voter-record reads without corresponding writes (potential exfiltration).
- **UEBA layer** — reuses HARD-6 UEBA baseline learner over the telemetry stream.
- **Coordinated device anomaly** — same anomaly across N devices in a T-minute window → possible coordinated attack.

## 8. Admin query console (hard-questions-fast surface)

New service crate `services/ecc-svc/crates/telemetry-query` + Django + LiveView + optional NL-to-query:

- **REST + gRPC query API**:
  - `GET /v1/telemetry/devices/{device_id}/timeline?from=&to=`
  - `POST /v1/telemetry/query` (structured query language)
  - `GET /v1/telemetry/geo/near?lat=&lon=&radius_m=&from=&to=`
  - `GET /v1/telemetry/officers/{officer_id}/sessions?from=&to=`
  - `GET /v1/telemetry/drops?severity=critical&from=&to=`
  - `GET /v1/telemetry/anomalies?class=&from=&to=`
- **Structured query DSL**: JSON body specifying `{device_ids, signal_classes, time_range, geo_polygon, aggregations, group_by}`; safe by construction; validated + rewritten before hitting the read replica.
- **Natural-language query bridge** (opt-in): LiteLLM-hosted local model translates admin English → structured DSL with per-query confirmation before execution. Guarded by SpiceDB `TELEMETRY_ANALYST` role.
- **Preset "hard question" library**:
  - "Which kits at station X came within Y m of a ballot box between T1 and T2?"
  - "Show every drop > 3g in county C in last N h."
  - "Officer with badge B: sessions + geo + reads/writes + anomalies."
  - "Kits below X% battery in region R: replacement dispatch status."
  - "Coordinated logout events in constituency C at time T."
  - "Longest gap between attestation heartbeats per kit."
- **LiveView dashboard** at `ecc-svc/ex/ecc_live/lib/ecc_live_web/live/telemetry_live.ex` — real-time timeline with 15 s refresh + PubSub push.
- **Django admin** at `ecc-svc/django/ecc-admin/apps/telemetry/` — deeper drill-down + evidence export for court.
- **Evidence export**: bundle per query with signature + Merkle inclusion proof + query text + result rows + timestamp → PDF + CSV + Parquet + tlog anchor for admissibility.

## 9. Integrity + tamper resistance

- **HMAC-signed payload** — every signal payload signed with per-device HMAC key from kit-attest.
- **Per-device prev_hash chain** — silent rewrite breaks the chain.
- **Daily Merkle snapshot per device** — anchored to P05.
- **Monotonic counter per device** — server rejects out-of-order signals.
- **Nonce replay window (5 min)** — per-device dedup.
- **Server-side re-signing** — ingestion server counter-signs the accepted signal with its own HSM signature before persistence.
- **Append-only trigger on signals table** — blocks UPDATE / DELETE at DB level.
- **Cross-check with independent signals** — GPS vs cell-tower vs Wi-Fi triangulation; drop signal must correlate with orientation-change signal within 200 ms.
- **Every admin query recorded** — `eel_telemetry_queries` captures who asked what when + what was returned + how many rows + evidence-export hash.

## 10. Privacy + PII posture

- **PII stripped on-device before egress** — voter IDs are hashed; officer names → SVID; addresses → district-level coarse geo when accuracy is poor.
- **Data minimisation** — raw 100 Hz accelerometer stream never leaves the device unless required for a court-ordered subpoena; 1 Hz downsample is authoritative for analytics.
- **Per-signal data class** — `eel_data_class_annotations` per signal type; log-scanner refuses to emit Tier-A/B fields.
- **Officer consent** — enrolment step includes signed acknowledgment of telemetry scope.
- **Right of erasure** — post-mission telemetry for an officer can be erased on ODPC-approved SAR/DSAR (except retention-mandated audit fields, which are pseudonymised).
- **Kenya DPA §26 minimisation + §41 automated processing** enforced at API + DB layers.

## 11. Honest caveats + external prerequisites

- **Device battery cost** — 100 Hz accelerometer sampling with local downsampling adds ~4-8 % daily battery draw. Mitigation: adaptive sampling (100 Hz only during active workflow, 10 Hz otherwise), configurable at kit registration.
- **Network cost** — every device pushes ~50-200 KB/day of signal data. Mitigation: batch + zstd + delta encoding.
- **PG partition growth** — 65 k kits × ~1 M signals/day = 65 B signals over the 3-year platform lifetime. Mitigation: monthly partitions + retention drop + TimescaleDB migration.
- **NL query model** — LiteLLM local model requires GPU or long CPU inference. Mitigation: preset library covers the top ~50 questions with sub-100 ms hit; NL bridge only for ad-hoc.
- **Officer consent + DPA §41** — automated flagging (drop, geo-fence, anomaly) must not trigger disciplinary action without human review; enforced at the alert-to-action boundary.
- **Court admissibility of accelerometer traces** — precedent is thin; expert witness testimony required per SCoK evidentiary framework.

## 12. Implementation split (4 work packages)

| WP | Scope |
|---|---|
| TELE-A | kit-sdk-flutter telemetry SDK + per-app instrumentation (all 13 Flutter apps) + Isar offline queue + hash chain + on-device redactor |
| TELE-B | field-mobile-svc/crates/telemetry-ingest + V052 migration (7 tables + append-only triggers) + Merkle snapshot cron + tlog anchoring |
| TELE-C | ecc-svc/crates/telemetry-query + REST + gRPC + Django admin + LiveView dashboard + evidence export bundler + NL bridge (opt-in) |
| TELE-D | ai-platform-svc/crates/telemetry-analytics + 9 anomaly detectors + UEBA integration + alert routing to ECC |

Total: 4 new crates + 1 SDK sub-module + 1 migration + 1 Django app + 1 LiveView + extensions to all 13 Flutter apps. Cross-references M16 (ECC) + M28 (AI monitoring) + HARD-6 (UEBA) + HARD-14 (behavioural biometrics) + HARD-16 (physical seal) + HARD-21 (audit chain).
