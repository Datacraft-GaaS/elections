# Queue transparency (M19)

Module M19 (`queue-transparency`) owned by `queue-svc`. Captures voter queue state from BVI kits and RTS stations in real time, computes station-level dwell times and service rates, publishes a live public turnout dashboard, fires predictive-overload alerts, and surfaces USSD queue-status queries for voters in areas with limited connectivity.

## Contents

1. What queue transparency does (vs M05 voter identification)
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

## 1. What queue transparency does (vs M05 voter identification)

- **M05** = authenticates the voter at the BVI kit and marks them as having voted.
- **M19** = consumes the events M05 and M08 produce, aggregates them into queue metrics (queue length, dwell time, service rate, station status), and makes those metrics public and actionable without exposing voter identity.

M19 has no write path for external parties. All metric records are computed from events; no manual entry.

## 2. Data plane

Four aggregates (per `services/queue-svc/crates/queue-api/src/models.rs`):

| Aggregate | Purpose | Data class |
|---|---|---|
| **StationQueueMetric** | station_id, recorded_at, queue_length_est, avg_dwell_s, service_rate_pph, throughput_today, status (NORMAL/SLOW/OVERLOADED/CLOSED) | D |
| **RegionalRollup** | region_id, recorded_at, constituency_count, active_stations, total_turnout_est, peak_queue_length | D |
| **OverloadAlert** | alert_id, station_id, fired_at, predicted_wait_min, severity, resolved_at | D |
| **UssdQueryLog** | query_id, msisdn_hash, station_code, queried_at, response_dwell_s | D |

`msisdn_hash` = SHA-256(E.164 number + daily salt); no plaintext MSISDN stored.

## 3. Ingest sources

| Source | Cadence | Adapter | Auth | Failure semantics |
|---|---|---|---|---|
| **M05 BVI kit events** (`eel.kit.voter_identified.v1`) | Real-time Kafka | Internal consumer | SVID mTLS | Lag alarm > 30 s; metrics degrade gracefully to last known |
| **M08 RTS station events** (`eel.rts.result_transmitted.v1`) | Real-time Kafka | Internal consumer | SVID mTLS | Same lag alarm |
| **Field clerk manual status** (station OPEN/CLOSED/INCIDENT) | On-demand | queue-api REST | JWT + SpiceDB `CLERK` role | Logged; overridden by next BVI event |
| **USSD gateway** (voter queries `*xxx#`) | Real-time | `integrations/ussd-gateway/adapter` | Gateway HMAC | Rate 5000/min; SHA-256 MSISDN hash stored only |

## 4. Pipeline

Files under `services/queue-svc/crates/queue-stream/src/`:

```
M05/M08 CloudEvent (eel.kit.voter_identified.v1)
      -> consumer group: queue-svc
      -> dwell time computation:
            dwell = identification_end_ts - queue_join_ts
            (queue_join_ts emitted by kit turnstile sensor or clerk scan-in)
      -> Kalman filter update on service-rate estimate per station
            (Welch and Bishop 2006, TR 95-041, UNC-CH)
            smooths noisy kit clock jitter without anchoring to wall-clock
      -> queue_length_est = arrivals_last_15min - departures_last_15min
      -> overload prediction: if predicted_wait_min > 60
            fire OverloadAlert (severity = WARN/CRIT)
            emit eel.queue.overload_alert.v1
      -> StationQueueMetric snapshot every 60 s per station
      -> RegionalRollup every 5 min
      -> emit:
            eel.queue.metric_updated.v1 (station granularity)
            eel.queue.regional_rollup.v1
      -> USSD query path (parallel):
            MSISDN received -> hash -> lookup station by home-ward
            -> respond dwell_s + status within 5 s
            -> log UssdQueryLog (hash only)
      -> publish metrics to P05 tlog snapshot at T+24h
         (daily aggregate; no individual dwell times)
```

Transactional outbox via `packages/eel-outbox-rs`; Debezium publishes to `eel.queue.v1`.

## 5. API surface

Contract: `packages/openapi-contracts/queue/public.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /queue/v1/stations` | Public | List all station metrics; CDN-cached 60 s |
| `GET /queue/v1/stations/{id}` | Public | Single station metric + alert history |
| `GET /queue/v1/regions` | Public | Regional rollup |
| `GET /queue/v1/alerts` | ECC operator (JWT) | Active overload alerts |
| `PATCH /queue/v1/stations/{id}/status` | Clerk (JWT + SpiceDB) | Set OPEN/CLOSED/INCIDENT |
| `GET /queue/v1/ussd/status` | USSD gateway (HMAC) | Voter-facing dwell query response |
| `GET /queue/v1/stations/{id}/history` | Commission admin | 24-h metric history |

All reads CDN-cached where possible. No personal voter data in any response.

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Public dashboard (Astro) | `services/queue-svc/astro/queue-portal/` | Live choropleth map, per-station status, turnout bar charts, WCAG 2.2 AA |
| ECC ops console (Phoenix LiveView) | `services/queue-svc/ex/queue_ops_web/` | Real-time overload alerts, acknowledgement, resource dispatch trigger |
| Clerk companion (field-mobile-svc M31) | Embedded widget | Station OPEN/CLOSED toggle; read-only dwell panel |

Languages: EN + SW; USSD response in EN + SW with KSL advisory for polling-day info clips.

## 7. Integrity + anti-tamper

- **Kalman filter traceability** -- per-station filter state (mean, variance) stored per snapshot; any manual override of state requires a commissioner signature and is logged as a `MANUAL_OVERRIDE` event.
- **No raw voter events in public API** -- the consumer strips voter_id from M05 events before writing to the metric store; the Kafka topic `eel.queue.v1` never contains voter-identifiable fields.
- **MSISDN privacy** -- USSD queries stored only as `SHA-256(msisdn + daily_salt)`; daily salt rotated at midnight EAT.
- **Metric immutability** -- `StationQueueMetric` rows are append-only; corrections create a new row with `amendment_of` FK, not an update.
- **P05 daily anchor** -- 24-hour aggregate (total turnout per station, no dwell distribution) anchored in transparency log; public can verify station closure times.
- **Alert idempotency** -- `OverloadAlert` deduplication via station + severity + 15-min window; no alert storm on repeated BVI events.
- **Partition tolerance** -- if Kafka is unreachable, the BVI kit continues recording locally; queue-svc degrades to last known metric rather than showing zero.

## 8. Kenya statute alignment

- **Elections Act s.72 (secrecy of vote)** -- M19 never exposes individual voter dwell times or identification timestamps; all metrics are aggregated.
- **Kenya Data Protection Act s.26 (minimisation)** -- MSISDN hashed; no personal attribute stored in metric records.
- **Elections Act s.54 (turnout reports)** -- real-time turnout dashboard satisfies statutory public-access obligation during polling day.
- **NIST SP 800-90B** -- not directly applicable to queue metrics, but the USSD daily salt rotation follows the entropy guidance for token generation to avoid linkage.

## 9. Honest caveats + external prerequisites

- Queue-length estimates depend on kit turnstile or clerk scan-in events. Many stations do not have turnstile hardware; clerk manual scan-in is the fallback. If clerks forget to scan, queue-length estimates are understated. The operational guide must make scan-in mandatory.
- Kalman filter parameters (process noise, observation noise) must be tuned on historical data from the 2017 and 2022 elections. If those datasets are not available from the Commission, the filter will use generic defaults and may over-smooth or under-smooth in the first hour.
- USSD gateway SLA: the 5-second response target depends on the gateway operator (Safaricom / Africa's Talking). Under election-day load this may degrade. The integration must specify a contractual SLA before go-live.
- Predictive wait-time alerts are advisory only. The model does not consider station capacity, booth count, or officer-to-voter ratio. False positives (CRIT alerts at stations that drain quickly) are expected; calibrate thresholds on pilot data.

## 10. Test coverage + operational runbooks

Coverage under `services/queue-svc/crates/queue-stream/tests/` and `crates/queue-api/tests/`:
- Unit: Kalman filter update, dwell computation, queue-length estimator, alert deduplication.
- Property (`proptest`): metric rows append-only; dwell time non-negative; MSISDN never stored raw.
- Contract (Pact): M05 event consumer; M08 event consumer; USSD gateway consumer.
- Integration (`testcontainers`): Kafka round-trip; PG snapshot; P05 tlog anchor.
- Load: p95 station metric endpoint < 100 ms under 2000 concurrent users (CDN pre-warm tested).
- Security: MSISDN hash linkage; USSD HMAC bypass; OWASP ZAP scan.

Runbooks under `docs/runbooks/`:
- `queue-svc-degradation.md` -- Kafka lag > 30 s; USSD gateway unreachable; metric stale > 5 min.
- `election-day-surge-response.md` -- CRIT alert triage, resource dispatch, queue-svc scale-out.
- `station-closure-confirmation.md` -- marking station CLOSED, evidence anchoring in P05.

Observability targets: `platform/observability/queue-svc/grafana-queue.json` covers queue_length_est p95, service_rate_pph distribution, overload_alert_rate, USSD query latency, and Kafka consumer lag. SLOs: 99.9% metric write success; station metric staleness < 120 s; USSD response < 5 s at p99; P05 daily anchor lag < 60 min.

Chaos scenarios under `chaos/scenarios/eel-queue-*.yaml`: Kafka partition leader election during metric burst; USSD gateway brownout under peak SMS load; Kalman filter divergence injection (artificial jitter spike).

Related: full spec at `docs/plans/modules/queue-transparency.md`; capability catalogue at `services/queue-svc/docs/CAPABILITIES.md`.
