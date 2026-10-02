# M44 turnout-forecasting-analytics

| field | value |
|---|---|
| M-number | M44 |
| Home service | ai-platform-svc (ADR-0020) |
| Owning work package | WP-4.5 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5%; 99.5% on public dashboard during polling hours |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M44-turnout-forecasting-analytics.md |

## Purpose

Pre-election turnout forecasting from CVR demographics, historical turnout, and macro signals (weather, transport, registration density). Election-day turnout tracking with variance alerts. Post-election analytical dataset for M41 post-election review and academic/journalist access. Turnout variance alerts feed M41 integrity review and M16 operational response per Strategy §XIX.1.

Strategy reference: §XIX.1 (Anomaly Detection), §XIX.3 (Post-Election Review). DPA 2019.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Public Turnout Dashboard (WCAG 2.2 AA) | Astro | services/ai-platform-svc/astro/turnout-dashboard/ |
| the Commission Internal Analytics Console | Phoenix LiveView | services/ai-platform-svc/ex/analytics_web/ |
| API | Rust (axum) | services/ai-platform-svc/crates/ai-platform-api/ |
| Model Training Pipeline | Python (async) | services/ai-platform-svc/py/turnout_forecasting/ |

Languages: EN + SW; public dashboard WCAG 2.2 AA; forecast methodology document in EN + plain-language SW.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| ForecastRelease | id, election_id, published_at, model_version, training_dataset_hash (SHA-256 of CVR extract), constituency_forecasts (FK list) | C | T2 |
| ConstituencyForecast | id, forecast_release_id, constituency_code, registered_voters, forecast_turnout_pct, interval_80_low, interval_80_high, interval_95_low, interval_95_high | C | T2 |
| HourlyTurnoutReport | id, polling_station_code, constituency_code, report_hour (7-17 EAT), voters_turned_out, submitted_at, presiding_officer_id | C | T2 |
| TurnoutVarianceAlert | id, constituency_code, alert_hour, observed_pct, forecast_pct, deviation_pct, sigma_deviation, alerted_at, acknowledged_by, acknowledged_at | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/ai-platform/turnout.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/forecasts/latest | 1 | public | n/a | latest pre-election forecast all constituencies |
| GET /v1/forecasts/{release_id} | 1 | public | n/a | specific forecast release |
| GET /v1/turnout/live | 5 (SSE) | public | n/a | election-day live turnout per constituency (SSE stream); updated every 30 min |
| GET /v1/turnout/alerts | 2 | JWT+SpiceDB (OPERATIONS_ANALYST/COMMAND_CENTRE) | n/a | active variance alerts (the Commission internal) |
| POST /v1/turnout/alerts/{id}/acknowledge | 2 | JWT+SpiceDB (OPERATIONS_ANALYST) | yes | acknowledge alert |
| GET /v1/datasets/post-election | 4 | JWT+SpiceDB (PUBLIC/JOURNALIST/ACADEMIC) | n/a | post-election constituency-level dataset; academic sub-county requires DPO approval |
| POST /v1/datasets/academic-request | 2 | JWT+SpiceDB (ACADEMIC) | yes | request academic sub-county dataset; triggers DPO review |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.analytics.forecast_published.v1 | emits | eel.analytics.turnout.v1 | operational | yes | at-least-once idempotent |
| eel.analytics.turnout_variance_alert.v1 | emits | eel.analytics.turnout.v1 | operational | yes | exactly-once |
| eel.analytics.post_election_dataset_published.v1 | emits | eel.analytics.turnout.v1 | operational | yes | at-least-once idempotent |
| eel.analytics.hourly_turnout_ingested.v1 | consumes | eel.field.clerk.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M02 BVR Back-End | internal | integrations/bvr/ | yes | CVR demographic aggregate extract; no individual records retained |
| M08 RTS Device | internal | integrations/rts-device/ | yes | hourly Presiding Officer turnout reports; missing reports trigger RO supervisor alert |
| M16 Election Command Centre | internal | integrations/ecc/ | yes | turnout anomaly feed as operational layer; circuit breaker |
| M41 Post-Election Review | internal | integrations/review/ | yes | anomaly-correlation JSON report within T+30; queued if M41 not ready |
| M50 By-Election Module | internal | integrations/by-election/ | yes | by-election historical turnout as additional baseline; optional |
| KMD Weather API | external | integrations/kmd/ | yes (stub) | county weather forecast for pre-election model; static seasonal-average fallback pre-loaded |
| P05 Merkle Log | P05 | integrations/merkle/ | yes | dataset publication hash anchoring |

## ML and AI

Model class: B1 (central analytics model, ADR-0018). Gradient-boosted ensemble (e.g. XGBoost) retrained each election cycle. Inputs: CVR demographic aggregates, historical turnout time series (2013/2017/2022 general + by-elections), KMD weather forecast, KURA road-accessibility index, voter density per station. Outputs: constituency-level turnout point estimate + 80%/95% prediction intervals.

Hard constraints:
- No individual voter data enters the model at any stage; only aggregate demographic summaries.
- Model version, training dataset hash, and feature list stored per release; pipeline reproducible from archived inputs.
- Forecast methodology document published alongside each release in plain-language EN + SW.
- Model not used for any decision in the vote-counting or tally pipeline (ADR-0018 hard red line).

## Dependencies

- Modules: M02, M08, M16, M41, M50
- Platform: P05
- Packages: eel-outbox-rs, eel-resilience-rs
- Python packages: see `services/ai-platform-svc/py/turnout_forecasting/pyproject.toml`

## Feature checklist

Pre-election forecasting:
1. FR-M44-001: ingest CVR demographic aggregates per constituency (age-band, gender, disability flag, registration date) from M02 BVR backend; individual CVR records purged after aggregation step; no individual voter data retained in M44 schema
2. FR-M44-002: ingest historical turnout per constituency: 2013, 2017, 2022 general elections plus by-elections from M50; stored as baseline time series per constituency code
3. FR-M44-003: ingest macro signals: KMD weather forecast per county (API), constituency-level road-accessibility index (KURA static dataset), registered-voter density per polling station
4. FR-M44-004: train constituency-level turnout forecast model (gradient-boosted ensemble, e.g. XGBoost); outputs per constituency: point estimate, 80% prediction interval (low/high), 95% prediction interval (low/high); retrained each election cycle
5. FR-M44-005: model training pipeline completes within 4 hours on updated CVR extract; pipeline idempotent and reproducible from archived inputs; training logs stored per release
6. FR-M44-006: forecast published at T-30, T-14, T-7 before election day; each publication assigned model_version and training_dataset_hash (SHA-256 of CVR extract used); each release archived indefinitely
7. FR-M44-007: forecast methodology document published alongside each release in plain-language EN + SW; documents training data sources, features, and any fallback sources used
8. FR-M44-008: KMD weather API unavailable fallback: static seasonal-average dataset pre-loaded per county; forecast proceeds; methodology document notes fallback source used; fallback flag stored on ForecastRelease
9. FR-M44-009: model back-test harness: run model against 2022 election data; MAE <= 5 percentage points vs published turnout per constituency; executed as offline pytest test at pipeline completion
10. FR-M44-010: model not used for any decision in vote-counting or tally pipeline; ADR-0018 hard constraint enforced by architectural gate at deployment; model output path limited to analytics endpoints only

Election-day tracking:
11. FR-M44-011: Presiding Officers submit hourly voter count per polling station via M08 RTS device; structured hourly count field in existing M08 report format; data flows to M44 aggregation pipeline
12. FR-M44-012: missing hourly report triggers RO supervisor alert within 5 minutes of submission deadline for that hour; alert routed via M16 Election Command Centre
13. FR-M44-013: constituency-level turnout aggregated hourly; aggregation pipeline completes within 10 minutes of Presiding Officer submission deadline for each hour
14. FR-M44-014: variance alert: observed turnout deviates > 2 SD or > 15 percentage points from forecast trajectory at any constituency at any hour mark; alert generated to M16 and the Commission Operations Analyst within 5 minutes
15. FR-M44-015: variance alert acknowledgement: Operations Analyst acknowledges alert with reason code; acknowledged_by (actor ID) and acknowledged_at (timestamp) recorded on TurnoutVarianceAlert record
16. FR-M44-016: public turnout dashboard updated every 30 minutes during polling hours (07:00-17:00 EAT); shows constituency-level turnout trajectory vs forecast; no station-level data published during polling hours
17. FR-M44-017: public dashboard: choropleth map (constituency level) with animated hourly update; bar chart vs forecast; download button (CSV/JSON); methodology document link; no login required
18. FR-M44-018: public dashboard SSE stream (/v1/turnout/live) sends constituency-level update events every 30 minutes; CDN caches 30-minute snapshots; origin p99 <= 2 s under 50,000 concurrent users
19. FR-M44-019: final per-station turnout published after polls close and RO Form 34A data received; reconciled against CVR registered voter count per station; discrepancies flagged for M41 review
20. FR-M44-020: low-registration constituencies flagged on public dashboard with widened prediction intervals and explicit "low sample" label; M41 notified automatically if variance alerts cluster in flagged constituencies

Post-election analytics:
21. FR-M44-021: post-election constituency-level dataset: turnout, demographic breakdown (age-band, gender), weather index, historical delta, anomaly flags; available for download at T+7 via registered data-access API
22. FR-M44-022: post-election dataset publication anchored to P05 Merkle Log; SHA-256 dataset hash published alongside download link; researcher can verify dataset against hash
23. FR-M44-023: journalist API: read-only; aggregated per constituency; no individual voter data; rate-limited; no registration required; response includes dataset_hash for verification
24. FR-M44-024: academic dataset API: sub-county breakdown on request with the Commission DPO approval; DPA 2019 s.30(1)(e) research lawful basis; data-sharing agreement required before access granted
25. FR-M44-025: academic dataset request workflow: applicant submits request; DPO review triggered; access granted within 5 working days of DPO approval; batch DPO review slots published in advance at T+7
26. FR-M44-026: anomaly-correlation report: cross-reference turnout anomaly flags with M41 post-election integrity events; structured JSON delivered to M41 Review Officer within T+30; queued if M41 anomaly-report ingest API not yet ready
27. FR-M44-027: anomaly-correlation report schema versioned; M41 consumes via registered data-access API; report includes constituency_code, alert_hour, sigma_deviation, linked M41 finding_ids if available

Data and compliance:
28. FR-M44-028: no individual voter data at any pipeline stage; only aggregate demographic summaries enter model and storage; CVR individual records purged after aggregation confirmed; enforced by data-flow audit
29. FR-M44-029: DPA 2019 s.26 individual data protection; academic sub-county dataset governed by DPA 2019 s.30(1)(e) data-sharing agreement; DPIA registered with ODPC
30. FR-M44-030: public dashboard WCAG 2.2 AA; EN + SW; responsive mobile layout; methodology document in EN and plain-language SW
31. FR-M44-031: system availability >= 99.5% on public dashboard during polling hours; >= 99.5% overall; SLO alert fires within 5 minutes of breach; internal analytics console SLO tracked separately

## Acceptance criteria

1. AC-M44-01: T-30 forecast published; methodology document live on public dashboard; point estimates and 80%/95% intervals correct for all constituencies; back-test MAE <= 5 percentage points on 2022 data; verified by back-test harness and UAT
2. AC-M44-02: election-day variance alert delivered to M16 and the Commission Operations Analyst within 5 minutes of trigger threshold being met; verified by integration test with simulated hourly report crossing threshold
3. AC-M44-03: public dashboard updated within 30 minutes of each hourly aggregation during polling hours; p99 <= 2 s under 50,000 concurrent users; CDN cache-hit ratio >= 95%; verified by k6 load test
4. AC-M44-04: post-election constituency-level dataset available at T+7; P05 Merkle hash published alongside download link; researcher successfully verifies dataset against hash; verified by integration test
5. AC-M44-05: academic sub-county dataset request triggers DPO review workflow; sub-county data not served without confirmed DPO approval; 403 returned on unapproved access attempt; verified by security test
6. AC-M44-06: model training pipeline completes within 4 hours on updated CVR extract; back-test MAE <= 5 percentage points confirmed at pipeline exit; verified by pipeline timing test
7. AC-M44-07: hourly constituency-level turnout aggregation completes within 10 minutes of Presiding Officer submission deadline; verified by integration test with simulated M08 batch
8. AC-M44-08: missing hourly report triggers RO supervisor alert within 5 minutes; verified by fault-injection test skipping one station submission
9. AC-M44-09: KMD weather API unavailable at T-7; static seasonal-average fallback activated; forecast published on schedule; methodology document updated to note fallback; verified by fault-injection test
10. AC-M44-10: anomaly-correlation JSON report delivered to M41 within T+30 post-election; report queued if M41 not yet ready; verified by integration test confirming delivery timestamp
11. AC-M44-11: public dashboard availability >= 99.5% during polling hours; SLO breach triggers alert within 5 minutes; verified by SLO dashboard report
12. AC-M44-12: WCAG 2.2 AA on public turnout dashboard and methodology document; EN + SW verified by language-switch smoke test; zero critical axe-core violations

## Applicable edge cases

EC-DAT-01: duplicate hourly turnout report from same station and hour with same idempotency key and body; idempotent ingestion; counted once in constituency aggregate; cached response returned
EC-DAT-02: duplicate hourly report with same idempotency key but different voter count; 422 `idempotency-key-reuse` returned; Operations Analyst alerted; original count retained
EC-DAT-03: out-of-order hourly report arrives after subsequent hours already aggregated; accepted; constituency aggregate recomputed for affected hours; variance alert re-evaluated
EC-DAT-05: poison hourly report (voters_turned_out > registered_voters for that station); rejected to DLQ with reason; constituency shows "data pending" on dashboard until corrected report received
EC-DAT-10: zero turnout at a constituency at any hour; valid state; no div-by-zero in variance or coverage computation; renders "0%" on public dashboard without error
EC-DAT-12: Debezium slot lag on hourly turnout aggregation table; WAL alert at 60 s; outbox retained; disk guard active; no data loss; variance alert delivery unaffected
EC-SEC-05: injection on all API input fields (constituency_code filters, dataset request parameters); parameterised queries only; verified by Semgrep, sqlx compile-time checks, and ZAP
EC-SEC-09: rate-limit evasion on journalist API and public dataset download; keyed by subject/IP-slash-64; dataset download rate-limited separately from dashboard reads; 429 returned with Retry-After
EC-SEC-10: academic sub-county dataset requested without DPO approval; 403 returned; audit log entry created; no partial data served
EC-SEC-16: secrets in logs/traces; no voter PII or individual CVR demographic fields in OTEL spans; log-scanner fixture in CI confirms clean traces
EC-IDN-02: JWT expires mid-request on the Commission internal analytics console; in-flight request completes; next request returns 401 with silent refresh prompt
EC-IDN-10: SpiceDB down during internal analytics console access; fail-closed; 503 problem+json returned; public read-only dashboard unaffected (no auth required)
EC-SCL-01: 50,000 concurrent public dashboard users during election-day peak; CDN caches 30-minute snapshots; origin p99 <= 2 s; cache-hit ratio >= 95% measured by k6 5-region test
EC-SCL-02: thundering herd at poll-close when final station counts published; CDN absorbs burst; dataset download rate-limited; the Commission internal analytics console scales separately behind auth
EC-SCL-04: consumer lag > 60 seconds on hourly turnout ingestion queue; alert fires; backpressure applied; no OOM; variance alert delivery SLA preserved by priority lane
EC-SCL-03: hot Kafka partition on high-frequency constituency updates during election day; stream-id spread applied at producer; lag alert fires; HPA scales consumers
EC-CON-03: Presiding Officer on 2G only; M08 device submits hourly report via SMS/USSD path; M44 ingestion pipeline accepts SMS-formatted structured report via M08 adapter
EC-OPS-05: KMD weather API unavailable at T-7; static seasonal-average fallback pre-loaded per county; forecast proceeds; methodology document updated to note fallback source used
EC-ML-01: forecast model underperforms in low-registration constituencies; prediction intervals widened for those constituencies; explicit "low sample" flag set on ConstituencyForecast; M41 notified if variance alerts cluster in flagged constituencies
EC-ML-04: model accuracy bias across age-band or gender demographic groups; per-group MAE computed in back-test harness; fairness report stored per model release; threshold violations block release
EC-ML-06: wrong or unsigned model artefact presented to training pipeline; cosign-verified at pipeline start; pipeline refuses unsigned model artefact; alert fires; previous model release remains active
EC-LEG-02: academic sub-county dataset served only with DPA 2019 s.30(1)(e) data-sharing agreement and confirmed DPO approval; both conditions checked at access gate; no data served without both
EC-HUM-06: screen readers on public dashboard choropleth and methodology document; full task completion at WCAG 2.2 AA; verified by axe-core and manual scripts
EC-ML-07: variance alert flood when a systemic M08 outage makes many constituencies appear to lag forecast; alerts deduplicated per constituency per hour, rate-limited, and ranked by sigma_deviation before M16 delivery; outage-correlated alerts tagged as data-gap not turnout anomaly
EC-HUM-08: EN + SW on all public surfaces; no string truncation on constituency names or percentage labels; responsive mobile layout verified by viewport test

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | forecast aggregation pipeline, variance alert threshold computation, hourly ingestion idempotency, dataset hash | nextest + pytest | M44-UNIT- |
| back-test | run model against 2022 election data; MAE <= 5 percentage points vs published turnout | pytest (offline) | M44-BACKTEST- |
| contract | M08 hourly report ingest, M16 alert push, M41 correlation report, P05 hash anchor | Pact | M44-PACT- |
| integration | T-30 forecast publish; election-day pipeline; variance alert; T+7 dataset publication | integration_test | M44-INT- |
| e2e | simulate polling hours; PO submits hourly; variance alert; public dashboard update | nextest e2e | M44-E2E- |
| security | academic sub-county access without DPO approval (deny); dataset hash tamper detection | ZAP + red team | M44-SEC- |
| a11y | WCAG 2.2 AA on public dashboard; EN + SW | axe-core | M44-A11Y- |
| load | 50,000 concurrent public dashboard reads during polling hours | k6 | M44-LOAD- |

Tags: `#[trace("FR-M44-009","AC-M44-02","EC-ML-01")]`

## Runbooks and dashboards

- `docs/runbooks/ai-platform-svc-degradation.md` - extend with turnout forecast pipeline failures and KMD fallback procedure
- `docs/runbooks/turnout-variance-alert-response.md` - new runbook; variance alert escalation; M16 coordination; the Commission Commissioner notification SLA
- Grafana dashboard: `eel-iems-44` (alert ruleset: `iems-44-alerts`)
- SLO alerts: variance alert delivery <= 5 min; public dashboard update <= 30 min; post-election dataset published by T+7

## Open questions

- CVR demographic aggregate extract: M02 team must confirm the aggregate schema and API authentication before T-270; extract must be privacy-preserving (no individual records).
- M08 RTS device hourly turnout field: M08 firmware must include the structured hourly count field; confirm timeline with M08 team (target T-90).
- KMD API access agreement: Kenya Meteorological Department API requires MoU; target T-180; static seasonal-average fallback covers any delay.
- Academic dataset DPO batch-review capacity: standard data-sharing agreement template and pre-published batch DPO review slots to be arranged by T-7 post-election.
