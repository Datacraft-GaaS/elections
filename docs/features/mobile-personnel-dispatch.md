# Mobile personnel dispatch (M33)

Module M33 (`mobile-personnel-dispatch`) owned by `field-mobile-svc`. Manages the deployment, check-in, and tracking of 250 000+ polling day personnel. A write-behind Kafka + Valkey cache absorbs 250k check-ins in 90 minutes. GPS + cell + Wi-Fi triangulation with anti-spoofing guards prevents ghost check-ins. Fifteen-minute pings maintain real-time presence. Payroll triggers are exactly-once. A voice-guided low-literacy workflow serves officers who cannot read.

## Contents

1. What mobile personnel dispatch does (vs related modules)
2. Data plane (schema)
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What mobile personnel dispatch does (vs related modules)

- **M16 HR / payroll** = authoritative officer roster; M33 reads assignments from M16 and writes attendance events back for payroll computation.
- **M33** = real-time deployment tracking: check-in at assigned station, 15-min presence pings, GPS anti-spoof, and payroll trigger at station close.
- **M32 Field support** = fault resolution; M33 co-location data can be referenced in M32 diagnostics but M33 does not write to M32.

M33 does not record votes or modify the voter roll. All officer personal data is encrypted at rest and never exposed to non-authorised consumers.

## 2. Data plane

Five aggregates (server-side PostgreSQL + Valkey write-behind):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **OfficerAssignment** | Pre-election assignment: officer_id, station_id, role, shift, expected_checkin_ts. | B |
| **CheckInEvent** | GPS-verified check-in: event_id (UUID7), officer_id, station_id, gps_lat, gps_lon, cell_lat, cell_lon, wifi_lat, wifi_lon, triangulated_lat, triangulated_lon, accuracy_m, spoof_flag (bool), ts_hlc. | A |
| **PresencePing** | 15-min heartbeat: ping_id, officer_id, station_id, ts_hlc, method (GPS/CELL/WIFI/MANUAL). | B |
| **AbsenceAlert** | No ping for >20 min: alert_id, officer_id, station_id, last_seen_ts, status (OPEN/RESOLVED). | B |
| **PayrollTrigger** | Exactly-once payroll event: trigger_id (UUID7), officer_id, station_id, hours_worked, role, ts_hlc, sent_to_m16 (bool). | A |

## 3. Ingest sources

| Source | Cadence | Format | Auth | Failure semantics |
|---|---|---|---|---|
| Officer device GPS | At check-in + every 15 min | OS location API | Device cert | Fallback to cell + Wi-Fi; manual if all fail |
| Cell tower position | At check-in + every 15 min | Network API (device) | Device cert | Fused with GPS + Wi-Fi |
| Wi-Fi positioning | At check-in + every 15 min | OS location API | Device cert | Fused; indoor stations use Wi-Fi primary |
| M16 assignment feed | Pre-election | Kafka event | SVID mTLS | Assignment cache refreshed; stale if >24 h before election |
| Manual check-in (county coordinator) | On demand | Django form | JWT + SpiceDB | Logged as MANUAL; used when officer has no smartphone |

## 4. Pipeline

```
250k check-ins in 90-minute window (07:00-08:30):
  officer opens dispatch app -> station QR scan
  -> device collects: GPS, cell tower position, Wi-Fi position
  -> triangulated_lat / triangulated_lon computed:
       weighted mean of 3 sources (GPS > cell > Wi-Fi by accuracy)
       accuracy_m = std of three readings
  -> anti-spoof check:
       if GPS vs cell distance > 500 m: spoof_flag = true; alert to coordinator
       if GPS vs Wi-Fi distance > 200 m (indoor threshold): spoof_flag = true
  -> CheckInEvent published to Kafka (write-behind via eel-outbox-rs)
  -> Valkey write-behind: officer_id -> station_id (TTL 12 h)
  -> Kafka consumer flushes to PostgreSQL in micro-batches (500 events)
  -> 250k events absorbed in <90 min at 3 000 events/s peak

15-min presence pings:
  app wakes (background service) every 15 min
  -> PresencePing published; Valkey TTL refreshed
  -> if no ping for >20 min: AbsenceAlert created;
       county coordinator notified; M32 ticket if kit also offline

voice-guided workflow (low-literacy):
  officer selects "Voice guide" on main screen
  -> TTS (NLLB SW / local SW TTS) reads each step aloud
  -> officer confirms each step with large tap target
  -> fallback: county coordinator calls and enters on officer's behalf (MANUAL)

payroll trigger at station close:
  presiding officer confirms station close via M31 EOD
  -> M33 receives eel.pollclerk.eod_submitted.v1 event
  -> hours_worked computed from CheckInEvent ts to EOD ts
  -> PayrollTrigger(trigger_id = UUID7) created
  -> exactly-once: idempotency key = officer_id + station_id + polling_date
  -> eel.dispatch.payroll_triggered.v1 emitted to M16
```

## 5. API surface

Contract: `packages/openapi-contracts/field-mobile/dispatch.yaml`. All write endpoints enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /dispatch/v1/checkins` | Officer device (cert) | Submit GPS check-in |
| `POST /dispatch/v1/pings` | Officer device (cert) | Submit 15-min presence ping |
| `GET /dispatch/v1/officers/{id}/status` | County coordinator (JWT) | Officer presence status |
| `GET /dispatch/v1/stations/{id}/roster` | County coordinator / IAB (JWT) | Station attendance roster |
| `GET /dispatch/v1/absence-alerts` | County coordinator (JWT + SpiceDB) | Open absence alerts |
| `PATCH /dispatch/v1/absence-alerts/{id}` | County coordinator | Resolve absence alert |
| `POST /dispatch/v1/payroll` | M16 (SVID mTLS) | Receive payroll trigger (idempotent) |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Officer dispatch Flutter app (Android) | `services/field-mobile-svc/flutter/dispatch/` | Check-in; ping; voice guide; assignment view |
| County coordinator dashboard (Phoenix LiveView) | `services/field-mobile-svc/ex/dispatch_web/` | Real-time station roster; absence alerts; spoof flags |
| National ops dashboard (Grafana) | `platform/observability/field-mobile-svc/grafana-dispatch.json` | 250k check-in progress; peak throughput; spoof alert rate |

Voice guide language: SW (primary); EN (secondary). Large tap targets (72dp minimum). WCAG 2.2 AA on all web surfaces. Android only for dispatch app (iOS not required for officer devices).

## 7. Integrity + anti-tamper

- **Write-behind Kafka + Valkey** -- check-in events are published to Kafka first; the Valkey cache serves real-time presence queries during the ingestion peak; PostgreSQL is the durable store; a Kafka broker failure causes in-flight events to queue on the producer until the broker recovers (no loss).
- **GPS + cell + Wi-Fi triangulation anti-spoof** -- three independent location sources are required for a non-flagged check-in; an officer who presents only a GPS reading that is >500 m from the cell position is flagged as SPOOF; the check-in is still recorded (to avoid blocking legitimate officers with poor GPS) but the county coordinator is alerted.
- **Exactly-once payroll trigger** -- the idempotency key `officer_id + station_id + polling_date` prevents duplicate payroll events from M31 EOD retries; a second event with the same key returns the cached ack.
- **Absence alert append-only** -- AbsenceAlerts cannot be deleted; a coordinator can RESOLVE an alert but the original open timestamp is preserved.
- **Manual check-in audit** -- every MANUAL check-in is logged with the coordinator's JWT identity; bulk MANUAL check-ins (>10 per coordinator per hour) trigger an anomaly flag to the national ops dashboard.

## 8. Kenya statute alignment

- **Elections Act s.37 (officer deployment)** -- check-in records provide the statutory evidence of officer presence at the assigned station.
- **DPA 2019 s.26 (minimisation)** -- GPS coordinates stored only for check-in and pings; no continuous tracking; coordinates purged 90 days after election.
- **Elections Act s.98 (payroll accuracy)** -- exactly-once PayrollTrigger with hours_worked derived from verified check-in timestamp provides an auditable payroll basis.
- **Labour Relations Act 2007** -- hours_worked in PayrollTrigger is the input to overtime computation in M16; M33 is responsible for accurate time data only.

## 9. Honest caveats + external prerequisites

- GPS accuracy in urban high-rise environments may be insufficient (<100 m) for the anti-spoof threshold; the triangulation algorithm weights Wi-Fi higher indoors but Wi-Fi positioning depends on an up-to-date Wi-Fi fingerprint map at each station, which must be collected during pre-election site visits.
- Valkey write-behind achieves the 250k/90-min throughput target only if the Kafka broker cluster is sized to at least 6 partitions on the check-in topic; the infrastructure team must confirm partition count before election day.
- The voice guide quality depends on the NLLB SW TTS model; NLLB's spoken Swahili quality for procedural instruction has not been independently evaluated; a human voice recording is the recommended fallback for critical instructions.
- MANUAL check-in by county coordinator is a fallback for officers without smartphones; it introduces coordinator-level authority risk; the bulk MANUAL alert is the detection mechanism but not a prevention mechanism.
- M16 payroll processing depends on the exactly-once PayrollTrigger being received; a Kafka partition failure after EOD could delay trigger delivery; M16 must have a reconciliation process to detect missing triggers within 24 h.

## 10. Test coverage + operational runbooks

Coverage under `services/field-mobile-svc/crates/dispatch-api/tests/` and the Flutter app tests: unit tests for triangulation algorithm, anti-spoof checker, absence alert timer, payroll trigger idempotency; property-based tests asserting spoof flag always set when GPS-cell distance >500 m; load tests (k6) for 3 000 check-ins/s for 90 min (250k total); contract tests (Pact) for M16 payroll trigger, M31 EOD event; integration tests with Valkey write-behind, Kafka producer, SoftHSM, and GPS/cell/Wi-Fi stub; e2e Flutter integration test covering check-in, 15-min ping, absence alert, and payroll trigger; chaos test for Valkey unavailable during peak (verifies Kafka durability).

Runbooks under `docs/runbooks/`:

- `dispatch-degradation.md` (create): Valkey unavailable; Kafka broker partition failure; GPS spoof spike; payroll trigger delay.
- `check-in-peak-runbook.md` (create): pre-election capacity check; Kafka partition sizing; Valkey memory allocation.

Dashboards: `platform/observability/field-mobile-svc/grafana-dispatch.json` (check-in rate, spoof alert rate, Valkey write-behind lag, absence alert count, payroll trigger delivery lag). SLO: 250k check-ins in 90 min; payroll trigger delivery within 2 h of EOD; absence alert response <30 min.

Manual check-in abuse detection: bulk MANUAL check-ins above 10 per coordinator per hour trigger an anomaly flag to the national operations dashboard and the IAB; the anomaly flag is immutable and cannot be cleared by the same coordinator; it requires an IAB officer to review and resolve.

Write-behind durability guarantee: Valkey is configured with AOF persistence (fsync=always in production) to ensure in-flight check-ins survive a Valkey restart within the 90-minute peak window; the Kafka consumer monitors the Valkey queue depth and alerts if it exceeds 50 000 entries (indicating PG write lag).

Related: `docs/plans/modules/mobile-personnel-dispatch.md`; `services/field-mobile-svc/docs/CAPABILITIES.md`.
