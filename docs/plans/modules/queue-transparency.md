# M19 queue-transparency

| field | value |
|---|---|
| M-number | M19 |
| Home service | queue-svc (ADR-0020) |
| Owning work package | WP-4.2 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M19-queue-transparency.md |

## Purpose

Real-time polling station queue monitoring: fuse data from kit check-in rates, computer vision headcounts, grievance signals, and USSD reports to estimate queue length and wait time at each station, surface disenfranchisement risk flags to ECC (M16) and citizens (M13), and generate s.14A extension recommendations when the queue will not clear before official close.

## Surfaces and languages

| surface | technology | path under services/queue-svc/ |
|---|---|---|
| Public queue map | Astro on packages/eel-web-astro | astro/queue-portal/ |
| Queue API | Rust (axum + utoipa) | crates/queue-api/ |
| Signal fusion worker | Rust | crates/queue-fusion/ |
| ECC feed | Elixir (Phoenix LiveView channel) | ex/queue-live/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| QueueSnapshot | snap_id, station_code, estimated_length, estimated_wait_min, confidence, sources, computed_at, disenfranchisement_risk | D | T2 |
| DisenfranchisementFlag | flag_id, station_code, risk_level, trigger_sources, s14a_recommended, created_at, acknowledged_by | C | T2 |
| S14ARecommendation | rec_id, station_code, close_time, estimated_clearance_time, generated_at, forwarded_to_ecc | C | T2 |

## APIs

OpenAPI file: packages/openapi-contracts/queue/public.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /queue/v1/snapshots | 1 signed-static | public | n/a | All station snapshots; CDN-cached 30 s |
| GET /queue/v1/snapshots/{station} | 2 live-per-user | public | n/a | Live snapshot for a station; 10 s TTL |
| GET /queue/v1/flags | 3 accredited | JWT+SpiceDB | n/a | Disenfranchisement flag feed for ECC |
| GET /queue/v1/s14a | 3 accredited | JWT+SpiceDB | n/a | s.14A recommendations |
| POST /queue/v1/reports | 2 live-per-user | public | yes | Citizen / party-agent queue report (USSD/WA) |
| GET /queue/v1/map | 1 signed-static | public | n/a | GeoJSON map of queue status by station |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.queue.snapshot_updated.v1 | emits | eel.queue.snapshot.v1 | analytics | yes | at-least-once idempotent |
| eel.queue.disenfranchisement_flagged.v1 | emits | eel.queue.flag.v1 | operational | yes | at-least-once idempotent |
| eel.queue.s14a_recommended.v1 | emits | eel.queue.s14a.v1 | operational | yes | exactly-once |
| eel.bvi.voter_identified.v1 | consumes | eel.bvi.identification.v1 | operational | yes | at-least-once idempotent |
| eel.grievance.case_created.v1 | consumes | eel.grievance.case.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| BVI back-end M06 | internal | integrations/bvi-backend/ | yes | check-in rate signal; circuit breaker |
| Grievance channels M23 | internal | integrations/grievance/ | yes | queue-complaint signal |
| ECC M16 | internal | integrations/ecc/ | yes | flag push; circuit breaker |
| Verifier app M13 | internal | integrations/verifier/ | yes | snapshot relay |
| Computer vision cameras (optional) | external | integrations/cv-cam/ | yes (stub) | CV headcount optional signal |

## ML and AI

Signal fusion model:
- Inputs: kit check-in rate, CV headcount (optional), citizen USSD/WA reports, grievance flags.
- Output: estimated queue length + wait time + disenfranchisement risk score.
- Model: lightweight regression + Kalman filter; no GPU required.
- s.14A recommendation: rule-based trigger (estimated clearance > official close time + configurable buffer).
- Model card required before production (managed by M28).

## Dependencies

- Modules: M08 (check-in rate source), M13 (public snapshot consumer), M16 (ECC flag consumer), M23 (grievance signal source)
- Platform: P05 (audit), Valkey (snapshot cache)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-web-astro

## Feature checklist

1. FR-M19-001: fuse check-in rate, CV headcount, USSD reports, grievance signals into queue snapshot per station.
2. FR-M19-002: compute estimated wait time and disenfranchisement risk score per station.
3. FR-M19-003: public queue map: station markers coloured by queue status (green/amber/red).
4. FR-M19-004: citizen queue report via USSD/WA; relay to signal fusion.
5. FR-M19-005: disenfranchisement flag: risk >= threshold triggers flag to ECC (M16) within 60 s.
6. FR-M19-006: s.14A recommendation: if estimated clearance > close time + buffer, generate recommendation; forward to ECC.
7. FR-M19-007: ECC live feed: Phoenix LiveView channel with 10 s refresh.
8. FR-M19-008: confidence score per snapshot; low-confidence snapshots clearly labelled.
9. FR-M19-009: configurable signal weights and thresholds per election cycle.
10. FR-M19-010: historical snapshot archive: every snapshot retained for post-election analysis.
11. Snapshot computed within 60 s of new signal arrival.
12. Disenfranchisement flag delivered to ECC within 60 s of threshold breach.
13. Public map load p95 <= 500 ms.
14. s.14A recommendation generated within 5 min of trigger condition.

## Acceptance criteria

AC-M19-01: snapshot computed within 60 s of simulated check-in spike (integration test).
AC-M19-02: disenfranchisement flag reaches ECC within 60 s (integration test).
AC-M19-03: s.14A recommendation generated when estimated clearance > close time + buffer (unit test).
AC-M19-04: USSD queue report incorporated into next snapshot (integration test).
AC-M19-05: public map loads p95 <= 500 ms at 100k concurrent (load test).
AC-M19-06: low-confidence snapshot clearly labelled (UAT).
AC-M19-07: 99.5% availability (SLO measurement).

## Applicable edge cases

**connectivity**
EC-CON-02: mobile-first map; CDN-cached.
EC-CON-04: BVI feed disconnect; last-known snapshot; stale labelled.

**time**
EC-TIM-07: s.14A close time moves; snapshot model recomputes with new close time.
EC-TIM-05: snapshot staleness; confidence drops; stale threshold alert.

**data**
EC-DAT-01: duplicate citizen report same session; deduplicated by session hash.
EC-DAT-06: conflicting signals (CV says 10, USSD says 200); fusion model confidence drops; labelled uncertain.

**security**
EC-SEC-09: rate-limit citizen reports; key by IP+/64; anti-spam.
EC-SEC-16: secrets in report text refused.

**ML**
EC-ML-01: signal fusion model drift (new station layout); recalibration trigger.
EC-ML-04: CV camera tampered; signal quarantined; fusion continues without CV.

**legal**
EC-LEG-08: s.14A recommendation is advisory only; ECC officer must confirm before action; recommendation logged separately from decision.

**scale**
EC-SCL-06: 40k stations x 10 s refresh; snapshot fan-out.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | signal fusion, wait-time estimator, s.14A trigger rule, confidence scorer | nextest | M19-UNIT- |
| property | disenfranchisement flag fires iff risk >= threshold; s.14A trigger deterministic | proptest | M19-PROP- |
| contract | M06 identification event (consumer); M23 grievance signal (consumer); M16 flag push (provider); M13 snapshot (provider) | Pact | M19-PACT- |
| integration | BVI double; grievance double; ECC double; Valkey snapshot cache | nextest + testcontainers | M19-INT- |
| e2e | check-in spike -> snapshot update -> disenfranchisement flag -> ECC alert -> s.14A recommendation | ExUnit + Playwright | M19-E2E- |
| ML | fusion accuracy on historical election data | pytest + MLflow | M19-ML- |
| load | 40k stations x 10 s refresh; public map 100k concurrent | k6 | M19-LOAD- |
| security | citizen report spam; rate-limit bypass; OWASP ZAP | ZAP + Semgrep | M19-SEC- |

## Runbooks and dashboards

- docs/runbooks/queue-svc-degradation.md (create): BVI feed disconnect; CV camera offline; s.14A recommendation escalation.
- platform/observability/queue-svc/grafana-queue.json: snapshot freshness per station, disenfranchisement flag rate, s.14A recommendation count, ECC push latency.
- platform/observability/queue-svc/slo-alerts.yaml: 99.5% burn rate; snapshot staleness > 120 s; flag delivery > 60 s.

## Open questions

- CV camera integration: the computer vision headcount source is marked optional and requires IEBC procurement sign-off on camera hardware at each station; design treats it as an optional signal and the module operates without it.
- Signal fusion weights: the relative weighting of USSD reports vs check-in rate vs CV is currently equal; IEBC data team needs to calibrate on historical election data before G3.
