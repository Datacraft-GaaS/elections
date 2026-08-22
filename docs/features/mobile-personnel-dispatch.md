# Mobile personnel dispatch (M33)

Module M33 (`mobile-personnel-dispatch`) owned by `field-mobile-svc`. Manages check-in and real-time tracking for ~250,000 election workers. Uses write-behind Kafka + Valkey to absorb 250k check-ins within 90 minutes of station opening. GPS + cell + Wi-Fi triangulation detects spoof attempts (>1 km delta flagged). 15-minute status pings confirm presence throughout the day. Payroll trigger fires exactly once on authenticated check-out. USSD path for feature phones and voice-guided workflow for low-literacy users. PWA fallback.

## Contents

1. What mobile personnel dispatch does (vs M32 field support)
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

## 1. What mobile personnel dispatch does (vs M32 field support)

- **M32** = field support engineers managing equipment faults and spares.
- **M33** = the broader election-worker workforce: clerks, presiding officers, security personnel, queue controllers. Check-in confirms physical presence; check-out triggers payroll. M32 is a subset of the M33 workforce.

M33 is the source of truth for worker deployment status consumed by ECC and HR/payroll.

## 2. Data plane

Four aggregates (per `services/field-mobile-svc/crates/dispatch-api/src/models.rs`):

| Aggregate | Purpose | Data class |
|---|---|---|
| **WorkerCheckIn** | check_in_id, worker_id, station_code, gps_lat, gps_lon, cell_id, wifi_bssid, spoofed (bool), channel (APP/USSD/PWA), checked_in_at | D |
| **PresencePing** | ping_id, worker_id, station_code, pinged_at, channel, response_within_15_min (bool) | D |
| **WorkerCheckOut** | check_out_id, worker_id, station_code, checked_out_at, supervisor_sig, payroll_trigger_id, payroll_idempotency_key | D |
| **PayrollTrigger** | trigger_id, worker_id, station_code, hours_worked, triggered_at, payroll_system_ref, exactly_once_key | D |

GPS coordinates stored only at check-in and check-out; not retained from 15-minute pings.

## 3. Ingest sources

| Source | Cadence | Adapter | Auth | Failure semantics |
|---|---|---|---|---|
| **App check-in** (Flutter) | Election morning (90-min window) | REST + Valkey write-behind | JWT + device cert | Write-behind absorbs burst; Kafka flush within 60 s |
| **USSD check-in** (`*xxx#`) | Same window | `integrations/ussd-gateway/adapter` | Gateway HMAC | Rate 10,000/min; offline queue if gateway drops |
| **15-min status ping** | Throughout polling day | App push response / USSD | JWT or gateway HMAC | Missed ping within 15 min logged as `ABSENCE_SIGNAL`; supervisor alerted |
| **Check-out** | Station close | App + supervisor confirmation | JWT + supervisor JWT | Payroll trigger idempotent; duplicate check-out rejected |
| **GPS/cell/Wi-Fi** | At check-in | Device location API | n/a | Delta > 1 km between GPS + cell_id triangulation: spoofed = true |

## 4. Pipeline

Files under `services/field-mobile-svc/mobile/personnel-dispatch/` and `crates/dispatch-api/src/`:

```
Election day morning: 250k workers check in over 90 min
      -> write-behind pattern:
            POST /dispatch/v1/checkin -> Valkey queue (instant ACK to client)
            background worker flushes to PG via Kafka within 60 s
            Debezium to eel.dispatch.v1 topic
      -> GPS + cell + Wi-Fi triangulation:
            delta = distance(gps_point, cell_id_centroid)
            if delta > 1 km: spoofed = true; supervisor alert fired
            WorkerCheckIn.spoofed persisted; ECC heatmap updated

Every 15 minutes throughout polling day:
      -> app sends ping (background push token acknowledgement)
      -> USSD path: automated IVR prompt for feature-phone workers
      -> PresencePing recorded
      -> if no response within 15 min: ABSENCE_SIGNAL event emitted
            supervisor + ECC console notified
            if 2 consecutive misses: automated escalation to polling officer

Station close: worker checks out
      -> supervisor confirms check-out (dual action)
      -> hours_worked computed from check_in to check_out
      -> PayrollTrigger created with exactly_once_key = worker_id + station_code + date
         (server rejects duplicate within 24 h window)
      -> payroll system integration: POST to HR/payroll REST (M-Pesa or Pesalink path)
      -> PayrollTrigger.payroll_system_ref recorded

Voice-guided workflow (low-literacy path):
      -> USSD IVR: audio prompts in EN + SW + top 3 regional languages
      -> keypad input only; no literacy required
      -> check-in confirmed via SMS receipt

Events emitted via packages/eel-outbox-rs:
      eel.dispatch.worker_checked_in.v1
      eel.dispatch.absence_signal.v1
      eel.dispatch.worker_checked_out.v1
      eel.dispatch.payroll_triggered.v1
```

Debezium to `eel.dispatch.v1`.

## 5. API surface

Contract: `packages/openapi-contracts/field-mobile/dispatch.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /dispatch/v1/checkin` | Worker (JWT) | Check in to station (write-behind, instant ACK) |
| `POST /dispatch/v1/ping` | Worker (JWT / push) | 15-minute presence ping |
| `POST /dispatch/v1/checkout` | Worker + supervisor (JWT dual) | Check out + payroll trigger |
| `GET /dispatch/v1/workers` | ECC / commission admin | Worker status board by station |
| `GET /dispatch/v1/workers/{id}/history` | Commission admin | Check-in history and pings |
| `GET /dispatch/v1/spoof-alerts` | ECC operator | Active GPS spoof alerts |
| `GET /dispatch/v1/absence-signals` | ECC / supervisor | Active absence signals |
| `GET /dispatch/v1/payroll/{id}` | HR admin | Payroll trigger status |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Flutter app (worker) | `services/field-mobile-svc/mobile/personnel-dispatch/` | Android 8+ / iOS 14+; voice-guided; USSD deep link; PWA fallback |
| PWA fallback | Same codebase (Flutter Web) | For workers without smartphones |
| USSD IVR | `integrations/ussd-gateway/dispatch-ivr/` | Feature-phone workers; audio-guided; EN + SW + 3 regional languages |
| ECC dispatch console (Phoenix LiveView) | `services/field-mobile-svc/ex/dispatch_web/` | Real-time worker status, spoof alerts, absence signals, station coverage map |
| HR admin (Django) | `services/field-mobile-svc/django/dispatch-admin/` | Payroll trigger audit, worker roster management |

## 7. Integrity + anti-tamper

- **Write-behind Kafka + Valkey** -- 250k simultaneous check-ins are absorbed by Valkey; background Kafka flush to PG ensures no check-in is lost even under PG write contention.
- **GPS spoof triangulation** -- delta between GPS point and cell_id centroid (from carrier cell-to-location table) flagged at >1 km; Wi-Fi BSSID provides a third reference; spoof flag stored immutably.
- **Exactly-once payroll trigger** -- `exactly_once_key = worker_id + station_code + date`; server rejects duplicate within 24-hour window via idempotency table; prevents double-payment.
- **Presence-ping immutability** -- `PresencePing` rows append-only; no ping can be inserted retroactively to cover an absence; timestamp is server-assigned from HLC.
- **Dual-action check-out** -- check-out requires both worker JWT and supervisor JWT; single-actor check-out rejected; prevents ghost check-out without physical confirmation.
- **No persistent GPS tracking** -- GPS stored only at check-in and check-out; 15-minute pings record only presence boolean, not location; minimises surveillance risk.

## 8. Kenya statute alignment

- **Elections Act s.14A (polling staff)** -- check-in record is the statutory attendance register for polling staff; P05-anchored for audit.
- **Kenya Data Protection Act s.26 (minimisation)** -- GPS coordinates stored only at check-in/check-out; ping records store only presence flag; no continuous location tracking.
- **Kenya Data Protection Act s.40 (automated decisions)** -- payroll trigger is automated but supervisor-confirmed; absence escalation is automated but officer-adjudicated.
- **Employment Act 2007 (wages)** -- PayrollTrigger exactly-once mechanism prevents both non-payment (lost check-out) and double-payment (duplicate check-out); hours_worked is the statutory basis for daily rate calculation.

## 9. Honest caveats + external prerequisites

- Cell-ID triangulation accuracy depends on carrier cell-to-location data. IEBC must procure a cell-ID dataset from Safaricom, Airtel, Telkom before go-live; without this, spoof triangulation degrades to GPS-only.
- Write-behind Valkey assumes Valkey is persistent (AOF or RDB). If Valkey restarts without persistence, in-flight check-ins in the 60-second flush window are lost. Valkey must be configured with AOF persistence in production.
- Payroll integration path (M-Pesa or Pesalink) is not specified in the M33 plan; the HR/payroll REST adapter endpoint must be confirmed by IEBC HR. The adapter is written to a templated interface; the live endpoint is a deployment configuration item.
- Voice-guided IVR is budgeted for EN + SW + 3 regional languages. Determining the 3 regional languages requires a distribution analysis of the worker roster; IEBC HR must provide the language profile before recording sessions.

## 10. Test coverage + operational runbooks

Coverage under `services/field-mobile-svc/crates/dispatch-api/tests/` and `mobile/personnel-dispatch/integration_test/`:
- Unit: write-behind flush, GPS spoof delta, exactly-once payroll key, dual-action check-out, ping immutability.
- Property (`proptest`): no duplicate payroll trigger; spoof flag immutable; ping timestamp server-assigned.
- Load: 250,000 concurrent POST /checkin within 90 min; p99 < 2 s; Valkey write-behind flush within 60 s.
- Contract (Pact): M32 engineer check-in consumer; ECC absence-signal consumer; HR payroll trigger consumer.
- Integration: USSD gateway stub; write-behind Valkey + PG; GPS spoof detection.
- Security: GPS spoof bypass; payroll duplicate; supervisor JWT forgery; OWASP ZAP.

Runbooks under `docs/runbooks/`:
- `dispatch-surge-response.md` -- Valkey capacity, Kafka lag, PG write bottleneck on election morning.
- `spoof-alert-triage.md` -- ECC review, supervisor notification, station investigation.
- `payroll-trigger-audit.md` -- exactly-once key review, duplicate detection, HR reconciliation.

Related: full spec at `docs/plans/modules/mobile-personnel-dispatch.md`; capability catalogue at `services/field-mobile-svc/docs/CAPABILITIES.md`.
