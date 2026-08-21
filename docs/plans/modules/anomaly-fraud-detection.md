# M26 anomaly-fraud-detection

| field | value |
|---|---|
| M-number | M26 |
| Home service | ai-platform-svc (ADR-0020) |
| Owning work package | WP-4.4 (docs/plans/05-wave-4-operations.md) |
| Size | XL |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M26-anomaly-fraud-detection.md |

## Purpose

Implement a defence-in-depth seven-layer detection stack where a rigging attempt must defeat every independent layer to succeed, ranging from real-time kit telemetry anomaly detection (Layer 1) through cross-station statistical monitoring (Layer 2), distributional forensics (Layer 3), cryptographic reproducibility checks (Layer 4), RLA linkage (Layer 5), cross-source triangulation (Layer 6), and human-adjudicated tip lines (Layer 7); no layer grants autonomous authority and every alert routes to a named human decision point.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| ECC Unified Alert Console | Phoenix LiveView | services/ai-platform-svc/ex/ecc_detection_web/ |
| IAB Statistical Auditor Dashboard | Phoenix LiveView | services/ai-platform-svc/ex/iab_web/ |
| Public Detection Report page | Astro | services/ai-platform-svc/astro/detection-public-site/ |
| API | Rust (axum) | services/ai-platform-svc/crates/ai-platform-api/ |
| ML worker (central models) | Python | services/ai-platform-svc/py/detection_worker/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| KitTelemetryPacket | id, kit_serial, station_code, auth_count, auth_success_rate, throughput_per_hour, failure_reason_distribution, battery_pct, lat/lon, network_state, storage_free_mb, temperature_c, reboot_count, manual_override_count, on_device_anomaly_score, packet_at, kit_signature | C | T2 |
| DetectionAlert | id, layer (L1-L7), severity, station_code, constituency_code, description, contributing_signals, anomaly_score, dimensional_scores, generated_at, assigned_analyst_id, resolution, resolution_note, resolved_at, merkle_node_id, is_public | B | T1-consensus |
| CrossSourceComparison | id, station_code, kiems_candidate_tallies, pvt_candidate_tallies, manual_count_tallies, party_agent_tallies, max_divergence_pct, layer6_alert_id, compared_at | C | T2 |
| DistributionalAnalysisResult | id, contest_id, method (BEBER_SCACCO_2BL/ROZENAS_SCATTER/KLIMEK_BIVARIATE), stations_analysed, stations_flagged, p_value_distribution, methodology_note, published_at, merkle_node_id | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/detection/anomaly-detection.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/detection/telemetry | internal gRPC | SVID mTLS (kit cert) | yes | ingest kit telemetry packet; 1/kit/min |
| GET /v1/detection/alerts | 2 | JWT+SpiceDB (ECC_ANALYST) | n/a | list active alerts across all layers |
| GET /v1/detection/alerts/{id} | 2 | JWT+SpiceDB (ECC_ANALYST) | n/a | alert detail with contributing signals |
| PATCH /v1/detection/alerts/{id}/resolve | 2 | JWT+SpiceDB (ECC_ANALYST) | yes | record resolution decision |
| POST /v1/detection/pvt-submission | 2 | JWT+SpiceDB (KEOG_ELOG) | yes | submit PVT batch from observer network |
| GET /v1/detection/cross-source/{station} | 2 | JWT+SpiceDB (ECC_ANALYST) | n/a | Layer 6 comparison for a station |
| GET /v1/detection/distributional | 1 | public | n/a | Layer 3 distributional analysis results |
| GET /v1/detection/cryptographic | 1 | public | n/a | Layer 4 hash-chain and reproducibility status |
| GET /v1/detection/canary | 1 | public | n/a | canary value history and current status |
| GET /v1/detection/report | 1 | public | n/a | post-election detection report |
| GET /v1/detection/model-cards | 1 | public | n/a | published model cards for all deployed models |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.detection.layer1_alert.v1 | emits | eel.detection.alert.v1 | operational | yes | at-least-once idempotent |
| eel.detection.layer2_alert.v1 | emits | eel.detection.alert.v1 | operational | yes | at-least-once idempotent |
| eel.detection.layer4_critical.v1 | emits | eel.detection.alert.v1 | operational | yes | exactly-once |
| eel.detection.layer6_divergence.v1 | emits | eel.detection.alert.v1 | operational | yes | at-least-once idempotent |
| eel.detection.layer7_convergence.v1 | emits | eel.detection.alert.v1 | operational | yes | at-least-once idempotent |
| eel.detection.alert_resolved.v1 | emits | eel.detection.alert.v1 | operational | yes | at-least-once idempotent |
| eel.detection.detection_report_published.v1 | emits | eel.detection.report.v1 | operational | yes | exactly-once |
| eel.rts.telemetry_packet.v1 | consumes | eel.rts.telemetry.v1 | operational | yes | at-least-once idempotent |
| eel.rts.form34a_captured.v1 | consumes | eel.rts.form34a.v1 | event-source | yes | exactly-once |
| eel.governance.rla_recount_required.v1 | consumes | eel.governance.rla.v1 | operational | yes | exactly-once |
| eel.grievance.tip_submitted.v1 | consumes | eel.grievance.tip.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M08 RTS Device / M10 RTS Back-End | internal | integrations/rts-telemetry/ | yes | circuit breaker; telemetry queued |
| M21 RLA Audit Tool | internal | integrations/rla/ | yes | Layer 5 degrades to manual if unavailable |
| M23 Grievance Channels | internal | integrations/grievance/ | yes | tip queue; retried on reconnect |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | alert log queued; anchored on reconnect |
| P06 HSM | P06 | integrations/hsm/ | yes | signing queue |
| M11 Public Portal | internal | integrations/portal/ | yes | fallback CDN for public report |
| KEOG / ELOG | external (MoU) | integrations/pvt-api/ | yes (stub) | Layer 6 degrades gracefully; flag "data unavailable" |
| IFES / Carter Center / EU EOM | external (MoU) | integrations/observer-report/ | yes (stub) | Layer 7 tip source; stub in dev |
| Social media monitoring | external (aggregated) | integrations/social-monitor/ | yes (stub) | D1/D4 inputs; no individual user data |

## ML and AI

- A6 on-device anomaly detection (Isolation Forest variant, INT8 <= 3 MB): embedded in kit firmware; M28 A6
- B3 central telemetry anomaly detection (LSTM + Isolation Forest): Layer 2 peer-comparison; M28 B3
- B5 turnout implausibility flag (statistical model): feeds Layer 2; M28 B5
- B6 transmission-gap pattern alert (threshold + trend): feeds Layer 1; M28 B6
- Layer 3 distributional models: last-digit uniformity (Beber and Scacco 2012), Rozenas scatter, Klimek bivariate; Python scipy/statsmodels; methodology published alongside every report; Benford Law applied only to derived quantities (Deckert, Myagkov, Ordeshook 2011 caveat prominently shown)
- All models governed by M28 model registry; no model may reach production without published model card and IAB 72-hour advance notification

## Dependencies

- Modules: M08, M10, M21, M23, M11, M27
- Platform: P05, P06
- Packages: eel-outbox-rs, eel-outbox-py, eel-resilience-rs, eel-resilience-py, eel-crypto-rs

## Feature checklist

1. FR-M26-001: kit telemetry stream; signed packet every 60 s; 17 fields including auth rate, throughput, GPS, overrides, temperature
2. FR-M26-002: on-device Isolation Forest (INT8, <= 5 MB); 5 anomaly dimensions; trained on prior-election baselines; thresholds: throughput > 3 sigma, KL-divergence > 0.5, geofence drift, > 3 overrides/10 min, unexpected reboot
3. FR-M26-003: kit generates Layer 1 alert signed and appended to Merkle chain; subtle in-kit indicator to Presiding Officer
4. FR-M26-004: ECC dashboard shows Layer 1 alerts within 2 min; colour by severity; one-click field-officer dispatch with GPS
5. FR-M26-005: central ensemble (Isolation Forest + One-Class SVM + Autoencoder) on 5 dimensions: turnout, auth-time distribution, rejected-ballot rate, biometric-first-try rate, manual-override rate
6. FR-M26-006: per-station peer comparison every 10 min; alert on top 0.5% single-dimension or top 1% two-dimension deviation
7. FR-M26-007: Layer 2 alert to ECC within 5 min; peer-comparison plot + dimensional scores; advisory flag on result pipeline
8. FR-M26-008: model recalibration at T-30 dry-run; analyst feedback loop to reduce alert fatigue
9. FR-M26-009: last-digit uniformity test (Beber and Scacco 2012) on station tallies >= 25 votes; chi-squared p < 0.05 flags station
10. FR-M26-010: turnout-vote-share scatter analysis (Rozenas 2017; Klimek et al 2012 PNAS); bivariate histogram; advisory only
11. FR-M26-011: Benford 2BL applied only to derived quantities; caveat on raw-count misapplication in every Layer 3 report
12. FR-M26-012: Layer 3 report published within 4 hours of all Form 34As received; simultaneously to public portal, IAB, KEOG/ELOG
13. FR-M26-013: no Layer 3 result triggers autonomous action; IAB reviews flagged stations; can request targeted recount per Elections Act s.80
14. FR-M26-014: Merkle-inclusion check for every Form 34A; missing inclusion proof = Layer 4 critical alert to SOC + IEBC Chair
15. FR-M26-015: hash-chain continuity check; any gap = Layer 4 critical alert
16. FR-M26-016: reproducibility check; re-run national aggregation from raw Form 34A JSON; any variance = Layer 4 critical alert published immediately
17. FR-M26-017: cryptographic canary published daily from T-30; canary-hash mismatch = Layer 4 critical alert + public announcement within 15 min
18. FR-M26-018: all Layer 4 alerts are public; published to portal and IAB within 15 min; no suppression mechanism
19. FR-M26-019: Layer 5 = RLA module (M21); its output is the Layer 5 signal
20. FR-M26-020: RECOUNT_REQUIRED from Layer 5 auto-creates Layer 5 critical alert cross-referenced with Layer 2 and Layer 3 flags
21. FR-M26-021: Layer 5 alerts published to public portal with statistical justification immediately
22. FR-M26-022: ingest PVT results from KEOG/ELOG via signed API; map PVT station codes to KIEMS codes; compare per-station tallies
23. FR-M26-023: Layer 6 alert triggers: > 2% divergence per constituency, > 0.5% at national level, > 10 votes/candidate per station
24. FR-M26-024: presiding-officer manual count cross-check; any discrepancy > 0 triggers Layer 6 flag
25. FR-M26-025: party-agent independent tally cross-check; any divergence > 0 votes/candidate triggers Layer 6 flag
26. FR-M26-026: Layer 6 alert includes divergence value, sources, station/constituency, recommendation; ECC Commander decides
27. FR-M26-027: Layer 7 ingests grievances from M23; observer field reports from KEOG/ELOG/IFES/Carter Center/EU EOM; media signals; whistleblower reports
28. FR-M26-028: signal aggregation; >= 3 independent sources, same issue type, same station/constituency = Layer 7 convergence alert; threshold configurable by IAB pre-election
29. FR-M26-029: Layer 7 adjudication; analyst marks each signal VERIFIED/UNVERIFIED/FALSE_POSITIVE; >= 2 verified = escalation to ECC Commander
30. FR-M26-030: ECC Commander decision: hold result, dispatch field officer, refer to DCI/ODPP, or clear with reason code; all decisions logged and published within 24 hours
31. FR-M26-031: Election Offences Act 2016 evidence-package generation for verified Layer 7 alerts with >= 2 verified sources; DCI referral within 6 hours
32. FR-M26-032: unified alert console; all seven layers; sorted by layer, severity, time; cross-layer correlation badge; filter by station/constituency/layer/severity
33. FR-M26-033: no layer grants autonomous ballot rejection, result annulment, or voter exclusion; every alert advisory until named human decision logged
34. FR-M26-034: all alert events and resolutions appended to Merkle audit chain
35. FR-M26-035: public-facing detection report published within T+4 hours of Form 34C; covers Layers 3, 4, 6, 7 detail; Layers 1 and 2 in aggregate
36. Live national telemetry map of all 55,393 stations; colour by anomaly score; geofence violation indicator
37. Layer 4 cryptographic status panel: real-time Merkle-inclusion progress bar; canary value display; any mismatch displayed in red with one-click public alert
38. DPA 2019: telemetry is station-level aggregates only at Layers 1-2; voter-level access only under logged investigation event
39. Model card published for every deployed model at T-30 per M28 governance rules
40. Red-team adversarial test at T-90; all seven layers tested simultaneously

## Acceptance criteria

1. AC-M26-01: Layer 1 alert from kit to ECC receipt <= 2 min; timestamp delta measurement
2. AC-M26-02: Layer 2 peer-comparison cycle <= 10 min per constituency; cycle log
3. AC-M26-03: Layer 4 Merkle-inclusion check covers 100% of Form 34As; reconciliation report
4. AC-M26-04: Layer 4 reproducibility check result = PASS (zero variance); reproducibility script output
5. AC-M26-05: Layer 6 PVT cross-check computed within 5 min of each PVT batch; comparison log
6. AC-M26-06: Layer 7 convergence alert generated within 5 min of third confirming signal; alert timestamp
7. AC-M26-07: public detection report published <= T+4 hours from Form 34C; publication timestamp
8. AC-M26-08: autonomous decisions affecting ballots or voters = 0; audit review by IAB
9. AC-M26-09: model cards published for all production models at T-30; registry timestamp vs deploy timestamp
10. AC-M26-10: WCAG 2.2 AA audit pass on ECC console and IAB auditor screens

## Applicable edge cases

**connectivity**
- EC-CON-01: kit telemetry offline; Layer 1 alert delayed; ECC sees "no telemetry" indicator distinct from "alert"
- EC-CON-03: 2G connectivity; telemetry packet compression; Layer 2 still computes from partial data

**timing**
- EC-TIM-03: Layer 4 canary mismatch under investigation; 15-min window before public announcement; "under investigation" message shown immediately

**data**
- EC-DAT-11: form34a count mismatch on reproducibility check; Layer 4 critical alert; no suppression path
- EC-DAT-15: KEOG/ELOG PVT batch unavailable; Layer 6 degrades gracefully; flag shown as "data unavailable" not "clear"

**security**
- EC-SEC-01: attempt to suppress Layer 4 alert via API; no suppression endpoint exists; attempt logged
- EC-SEC-02: false Layer 6 PVT batch submitted; station-code mapping rejects unrecognised codes
- EC-SEC-09: adversary studies detection algorithm to evade all 7 layers simultaneously; operationally infeasible due to independent accountability chains

**legal**
- EC-LEG-05: Layer 3 distributional result used as petition evidence; methodology published; Benford caveat included; IAB statistician as expert witness

**operations**
- EC-OPS-04: ECC analyst alert fatigue from Layer 2 false positives; snooze mechanism with reason code for repeated patterns per station; T-30 recalibration
- EC-OPS-07: central detection cluster degraded during election day; fallback to Layer 1 kit-side alerts only; Layer 2-6 marked "unavailable" not "clear"

**ML**
- EC-ML-01: Isolation Forest model drift on election day vs training baseline; recalibration dry-run at T-30; analyst feedback mechanism
- EC-ML-02: Layer 3 distributional test applied inappropriately to small-count stations (< 25 votes); guard: test only on stations with count >= 25

**scale**
- EC-SCL-01: 55,393 simultaneous telemetry packets on peak; worker pool auto-scales; SLO on Layer 2 cycle maintained

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Isolation Forest inference, last-digit chi-squared, Merkle-inclusion check, convergence threshold logic, divergence computation | nextest + pytest | M26-UNIT- |
| property | alert-log append idempotency, cross-source divergence invariants | proptest | M26-PROP- |
| contract | kit telemetry ingestion, PVT API, RLA event consumer, grievance event consumer | Pact | M26-PACT- |
| integration | full Layer 1 pipeline; Layer 6 PVT ingestion; Layer 7 tip aggregation with synthetic signals | testcontainers | M26-INT- |
| e2e | simulated election with injected anomalies at all 7 layers; verify all layers fire; verify zero false autonomous actions | Playwright + synthetic-kit | M26-E2E- |
| statistical | Layer 3 outputs vs known synthetic fraud patterns; false-positive calibration on prior-election data | pytest | M26-STAT- |
| security | attempt to suppress Layer 4 alert; attempt to inject false Layer 6 PVT batch; attempt to modify resolved alert | ZAP + cargo-fuzz | M26-SEC- |
| a11y | ECC console and IAB screens | axe-core + manual | M26-A11Y- |
| load | 55,393 simultaneous telemetry packets; Layer 2 computation under full load | k6 | M26-LOAD- |
| chaos | central detection cluster partial failure; verify Layer 1 continues; Layers 2-6 marked degraded not clear | Chaos Mesh | M26-CHAOS- |

Tags: `#[trace("FR-M26-033","AC-M26-08","EC-ML-01")]`

## Runbooks and dashboards

- `docs/runbooks/ai-platform-svc-degradation.md` - extend with detection-stack partial degradation paths
- `docs/runbooks/layer4-canary-mismatch-response.md` - new runbook; 15-min window, "under investigation" message, SOC escalation steps
- `docs/runbooks/detection-red-team-response.md` - response protocol when all 7 layers fire simultaneously
- Grafana dashboard: `platform/observability/ai-platform-svc/detection-stack-dashboard.json`
- SLO alerts: `platform/observability/ai-platform-svc/detection-slo-alerts.yaml` (Layer 1 <= 2 min, Layer 4 coverage 100%, report <= T+4 h)

## Open questions

- KEOG/ELOG PVT API authentication schema and data format require MoU sign-off before integration sprint; build on signed stub until agreed.
- 2013/2017/2022 election baseline data: IEBC historical archive access must be granted to Datacraft by T-270 for ML training; timeline risk.
- Layer 3 methodology: Kenyan Electoral Commission statistician peer review of Beber-Scacco and Klimek implementations recommended before T-90 public deployment.
- Observer mission MoUs (IFES, Carter Center, EU EOM) must specify Layer 7 API authentication before integration sprint.
