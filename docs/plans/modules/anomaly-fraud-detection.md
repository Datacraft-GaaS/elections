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

Layer 1: Kit-side self-monitoring (real-time per-kit):

1. FR-M26-001: Kit telemetry stream: signed packet every 60 seconds per kit; 17 fields: auth count, auth success rate, throughput per hour, failure-reason distribution, battery level, GPS coordinates, network state, storage free, temperature, reboot count, manual-override count, on-device anomaly score, kit_signature.
2. FR-M26-002: On-device Isolation Forest (INT8, <= 5 MB): anomaly score on 60-minute rolling window; trained on prior-election baselines; five alert thresholds: throughput > 3 sigma, KL-divergence > 0.5 from hourly baseline, geofence drift outside 5 km tolerance, > 3 manual overrides in 10 minutes, unexpected reboot.
3. FR-M26-003: Kit-side Layer 1 alert: signed; appended to Merkle chain on threshold crossing; transmitted to ECC; subtle in-kit indicator to Presiding Officer; no voter-visible alarm displayed.
4. FR-M26-004: ECC receives Layer 1 alert within 2 minutes; colour-coded by severity; one-click field-officer dispatch with GPS coordinates and current telemetry.

Layer 2: Cross-station statistical monitoring (real-time per-constituency):

5. FR-M26-005: Central unsupervised ensemble (Isolation Forest + One-Class SVM + Autoencoder) on five station-level dimensions: turnout, authentication-time distribution, rejected-ballot rate, biometric-first-try-success rate, manual-override rate; trained on prior-election station-level patterns.
6. FR-M26-006: Per-station peer comparison every 10 minutes; alert on any station in top 0.5% of peer-deviation on any single dimension or top 1% on any two dimensions simultaneously within its constituency.
7. FR-M26-007: Layer 2 alert to ECC within 5 minutes of trigger; includes peer-comparison plot, dimensional scores, contributing features; advisory flag on result-publication pipeline (does not hold result without ECC Commander decision).
8. FR-M26-008: False-positive tuning: model recalibrated at T-30 dry-run on synthetic data; ECC analyst feedback mechanism during election day; analyst can snooze repeated false-positive patterns for a station with reason code.
9. FR-M26-009: All 55,393 stations included in Layer 2 peer comparison; no station excluded by default configuration.

Layer 3: Statistical distributional analysis (post-close per-station):

10. FR-M26-010: Last-digit uniformity test (Beber and Scacco 2012, AJPS 56(1)): applied to final digits of per-station candidate tallies where count >= 25; chi-squared test at p < 0.05 flags station; methodology published alongside results.
11. FR-M26-011: Turnout-vote-share scatter analysis (Rozenas 2017; Klimek et al 2012, PNAS): bivariate histogram on constituency-level turnout vs winner-vote-share; linear correlation and cluster analysis published; advisory only.
12. FR-M26-012: Benford 2BL applied only to derived quantities (margins, ratios) where mathematical preconditions hold; caveat on raw-count misapplication prominently included in every Layer 3 report; per Deckert, Myagkov, Ordeshook 2011.
13. FR-M26-013: Layer 3 report published within 4 hours of all Form 34As received; delivered simultaneously to public portal, IAB, and KEOG/ELOG.
14. FR-M26-014: No Layer 3 result triggers autonomous action; IAB reviews all flagged stations; can request targeted recount per Elections Act s.80.

Layer 4: Cryptographic and reproducibility checks (post-close whole-tally):

15. FR-M26-015: Merkle-inclusion check for every Form 34A: verify hash included in constituency Form 34B Merkle root and that root in Form 34C root; missing inclusion proof = Layer 4 critical alert to SOC + IEBC Chair.
16. FR-M26-016: Hash-chain continuity check: verify unbroken sequence of Merkle nodes from station open to Form 34A publish; any gap = Layer 4 critical alert.
17. FR-M26-017: Reproducibility check: re-run full national aggregation from raw Form 34A JSON; compare to published Form 34C bit-identically; any variance = Layer 4 critical alert published immediately.
18. FR-M26-018: Cryptographic canary: Datacraft publishes known-good canary transactions in RTS pipeline daily from T-30; Merkle-anchored with published timestamp; canary-hash mismatch = Layer 4 critical alert + public announcement within 15 minutes.
19. FR-M26-019: All Layer 4 alerts are public: published to portal and IAB within 15 minutes of detection; no suppression mechanism exists or can be added via API.
20. FR-M26-020: Layer 4 hash-chain continuity and Merkle-inclusion checks cover all 55,393 stations; completed within 60 minutes of Form 34C; check progress visible in real time on Layer 4 status panel.

Layer 5: RLA linkage:

21. FR-M26-021: Layer 5 is the RLA module (M21); its CONFIRMED or RECOUNT_REQUIRED output is the Layer 5 signal in the detection stack.
22. FR-M26-022: RECOUNT_REQUIRED event from Layer 5 automatically creates Layer 5 critical alert cross-referenced with any Layer 2 and Layer 3 flags at affected stations.
23. FR-M26-023: Layer 5 alerts published to public portal immediately with statistical justification; no delay permitted.

Layer 6: Cross-source triangulation (real-time whole-country):

24. FR-M26-024: Ingest PVT results from KEOG/ELOG via signed API; map PVT station codes to KIEMS station codes; compare candidate tallies per station.
25. FR-M26-025: Layer 6 alert triggers: divergence > 2% at any constituency; divergence > 0.5% at national level; KIEMS result differs from PVT by > 10 votes per candidate at any station.
26. FR-M26-026: Presiding-officer manual count cross-check: cross-reference kit-entered tallies with separately uploaded manual count captured at close on a separate tablet; any discrepancy > 0 triggers Layer 6 flag.
27. FR-M26-027: Party-agent independent tally cross-check: if party agent submits independently tallied Form 34A via party-agent portal, cross-reference against KIEMS tally; divergence > 0 votes per candidate triggers Layer 6 flag.
28. FR-M26-028: Layer 6 alert includes: divergence value, diverging sources, affected station/constituency, recommendation (hold pending review or proceed with explanation); ECC Commander decides; recommendation is advisory only.
29. FR-M26-029: Layer 6 degrades gracefully without PVT input; "data unavailable" flag shown in Layer 6 panel; no false clear; manual count and party-agent tally checks remain active.
30. FR-M26-030: PVT divergence computed within 5 minutes of each signed KEOG/ELOG batch submission.

Layer 7: Human-adjudicated tip line (real-time all-cycle):

31. FR-M26-031: Ingests signals from: M23 grievance channels (all six intake modes); observer field reports from KEOG, ELOG, IFES, Carter Center, EU EOM via signed reporting API; media investigative signals (manual ECC analyst entry); whistleblower reports from M23 protected channel.
32. FR-M26-032: Signal aggregation: >= 3 independent sources reporting same issue type at same station or constituency generates Layer 7 convergence alert; threshold configurable by IAB pre-election via IEBC Chair HSM-signed config change.
33. FR-M26-033: Layer 7 adjudication: ECC analyst marks each contributing signal VERIFIED/UNVERIFIED/FALSE_POSITIVE; >= 2 verified signals escalates to ECC Commander.
34. FR-M26-034: ECC Commander decision options: hold result pending field investigation; dispatch field officer; refer to DCI/ODPP; clear with reason code; all decisions logged and published within 24 hours of alert close.
35. FR-M26-035: Verified Layer 7 convergence alert with >= 2 independent verified sources auto-generates evidence package (via M23) for DCI referral within 6 hours.

Cross-layer:

36. FR-M26-036: Unified alert console: all seven layers in single dashboard; sorted by layer, severity, and time; cross-layer correlation badge on station appearing in multiple layers simultaneously; auto-elevated to CRITICAL when station appears in three or more layers.
37. FR-M26-037: No layer grants autonomous ballot rejection, result annulment, or voter exclusion; every alert is advisory until a named human decision is made and logged.
38. FR-M26-038: All alert events and resolutions appended to Merkle audit chain; alert_resolved events signed and anchored; IAB can verify the complete alert lifecycle.
39. FR-M26-039: Public detection report published within T+4 hours of Form 34C; includes Layer 3 distributional analysis, Layer 4 reproducibility result, Layer 6 divergence summary, Layer 7 convergence summary; Layers 1 and 2 published in aggregate.
40. FR-M26-040: Cross-layer composite alert: station appearing in Layers 1, 2, and 7 simultaneously auto-elevated to CRITICAL composite alert; ECC Commander notified immediately via push and on-screen badge.

NFRs as requirements:

41. FR-M26-041: Layer 1 kit alert to ECC receipt <= 2 minutes under normal load; measured by timestamp delta in every telemetry cycle.
42. FR-M26-042: Layer 2 peer-comparison cycle <= 10 minutes per constituency per cycle on election day at full 55,393-station load.
43. FR-M26-043: Layer 7 convergence alert generated within 5 minutes of the third confirming independent signal.
44. FR-M26-044: Central detection cluster availability >= 99.95% during T-7 to T+4; partial failure degrades each layer to "layer unavailable" not "clear"; Layer 1 kit-side alerts continue independently.
45. FR-M26-045: WCAG 2.2 AA on ECC console and IAB auditor-facing screens; independent audit before T-7 deploy; analysts with disabilities must complete all alert workflows.
46. FR-M26-046: DPA 2019: telemetry contains station-level aggregates only at Layers 1-2; voter-level authentication data accessed only under a logged investigation event; audit log of all voter-level accesses available to IAB.
47. FR-M26-047: Model governance: every model requires IAB 72-hour advance notification and published model card before production deployment; M28 model registry gates deployment.
48. FR-M26-048: Model recalibration dry-run at T-30 on synthetic election data; analyst feedback incorporated before T-7 production deploy.

APIs:

49. FR-M26-049: POST /v1/detection/telemetry: kit-cert mTLS; 1 packet per kit per minute; idempotent write; ingest to time-series store.
50. FR-M26-050: GET /v1/detection/alerts: ECC_ANALYST JWT; list active alerts across all layers; filterable by layer, severity, station, constituency.
51. FR-M26-051: GET /v1/detection/alerts/{id}: alert detail with contributing signal IDs, dimensional scores, and resolution history.
52. FR-M26-052: PATCH /v1/detection/alerts/{id}/resolve: ECC_ANALYST JWT; record resolution decision (CLEARED/ESCALATED/HELD/REFERRED); idempotent; decision published within 24 hours of Layer 7 close.
53. FR-M26-053: POST /v1/detection/pvt-submission: KEOG_ELOG JWT; signed PVT batch; station-code mapping validated on receipt.
54. FR-M26-054: GET /v1/detection/cross-source/{station}: ECC_ANALYST JWT; Layer 6 comparison for one station showing all available source tallies.
55. FR-M26-055: GET /v1/detection/distributional: public; Layer 3 distributional analysis results with methodology disclosure.
56. FR-M26-056: GET /v1/detection/cryptographic: public; Layer 4 hash-chain and reproducibility status in real time.
57. FR-M26-057: GET /v1/detection/canary: public; canary value history and current status; any mismatch shown immediately.
58. FR-M26-058: GET /v1/detection/report: public; post-election detection report with Merkle proof of publication timestamp.
59. FR-M26-059: GET /v1/detection/model-cards: public; model cards for all deployed models including training data source, eval metrics, and known limitations.

Events:

60. FR-M26-060: Emit eel.detection.layer1_alert.v1 on Layer 1 threshold crossing; signed; at-least-once idempotent.
61. FR-M26-061: Emit eel.detection.layer2_alert.v1 on Layer 2 peer-comparison trigger; signed; at-least-once idempotent.
62. FR-M26-062: Emit eel.detection.layer4_critical.v1 on any Layer 4 check failure; signed; exactly-once.
63. FR-M26-063: Emit eel.detection.layer6_divergence.v1 on Layer 6 alert trigger; signed; at-least-once idempotent.
64. FR-M26-064: Emit eel.detection.layer7_convergence.v1 on third confirming independent signal; signed; at-least-once idempotent.
65. FR-M26-065: Emit eel.detection.alert_resolved.v1 on analyst resolution decision; signed; at-least-once idempotent.
66. FR-M26-066: Emit eel.detection.detection_report_published.v1 on public report publication; signed; exactly-once.

UI screens:

67. FR-M26-067: Layer 1 telemetry map: live national map of all 55,393 stations; colour by anomaly score; click-through to station telemetry detail; geofence violation indicator; no station excluded from map.
68. FR-M26-068: Layer 2 peer comparison chart: constituency selector; multi-dimensional radar chart per station; outlier stations highlighted; historical baseline overlay; drill-down to dimensional scores.
69. FR-M26-069: Layer 3 distributional analysis dashboard: last-digit distribution bar chart for selected contest; choropleth of flagged stations; methodology disclosure toggle; Benford caveat prominently shown.
70. FR-M26-070: Layer 4 cryptographic status panel: real-time Merkle-inclusion check progress bar; canary value display; reproducibility check status; any mismatch displayed in red with one-click public alert button.
71. FR-M26-071: Layer 6 cross-source comparison table: per-station KIEMS tally, PVT tally, manual count, party-agent tally, divergence percentage; colour by divergence magnitude; filter for divergences above threshold.
72. FR-M26-072: Layer 7 convergence alert detail: all contributing signals with source, timestamp, GPS, verification status; analyst VERIFIED/UNVERIFIED/FALSE_POSITIVE controls; ECC Commander escalation button; evidence-package preview.

## Acceptance criteria

AC-M26-01: Layer 1 alert from kit to ECC receipt <= 2 minutes; timestamp delta measurement in integration test; ECC Commander sign-off.
AC-M26-02: Layer 2 peer-comparison cycle <= 10 minutes per constituency on election day at full 55,393-station load; cycle log confirms no cycle exceeds 10 minutes.
AC-M26-03: Layer 4 Merkle-inclusion check covers 100% of Form 34As received; reconciliation report shows zero unchecked forms; Datacraft SOC sign-off.
AC-M26-04: Layer 4 reproducibility check result = PASS (zero variance) on synthetic and real election aggregation data; reproducibility script output attached to release note.
AC-M26-05: Layer 6 PVT cross-check computed within 5 minutes of each signed KEOG/ELOG batch; comparison log shows latency per batch.
AC-M26-06: Layer 7 convergence alert generated within 5 minutes of the third confirming independent signal; alert timestamp verified in integration test with injected synthetic signals.
AC-M26-07: Public detection report published <= T+4 hours from Form 34C; publication timestamp logged and signed; IEBC Chair review.
AC-M26-08: Autonomous decisions affecting ballots or voters = 0 throughout election; IAB audit review of all alert resolutions confirms; any automated action is advisory only.
AC-M26-09: Model cards published for all production models at T-30; registry timestamp before deploy timestamp for 100% of models; Datacraft AI Lead sign-off.
AC-M26-10: WCAG 2.2 AA independent audit pass on ECC console and IAB auditor-facing screens; screen-reader full-task-completion verified for all seven alert workflow paths.
AC-M26-11: Red-team adversarial test at T-90 confirms all seven layers fire on simultaneous injected anomalies; no layer bypassed; test report published to IAB.
AC-M26-12: Central detection cluster available >= 99.95% during T-7 to T+4; SLO measurement with burn-rate alert; partial failure degrades each layer independently.
AC-M26-13: No suppression endpoint for Layer 4 alerts exists; attempt to suppress via API returns 404 or 405; security test confirms; ZAP scan shows no suppression path.
AC-M26-14: Layer 3 report delivered to IAB, KEOG/ELOG, and public portal simultaneously within 4 hours of last Form 34A; delivery timestamps from all three destinations logged.
AC-M26-15: All alert events anchored to Merkle chain; Merkle reconciliation script confirms 100% coverage at T+1; no alert missing from chain.
AC-M26-16: ECC Commander decisions on Layer 7 alerts logged and published within 24 hours of alert close; publication log cross-referenced with alert timestamps; IAB review.

## Applicable edge cases

EC-CON-01: Kit telemetry offline during connectivity outage; ECC sees "no telemetry" indicator visually distinct from "alert"; Layer 1 alerts resume on reconnect; no layer marked clear on missing data.
EC-CON-03: 2G connectivity on election day; telemetry packet compression (CBOR+zstd); Layer 2 computes from partial data; partial-data flag shown in Layer 2 panel.
EC-CON-04: Site partition between primary and mirror detection cluster; Patroni fencing prevents split-brain; MirrorMaker lag alert; RPO <= 5 minutes; no layer shows stale clear state.
EC-CON-11: Kafka ISR shrink; detection worker outbox accumulates telemetry and alert events; drains when brokers recover; no event lost; no layer marked clear during accumulation.
EC-TIM-03: Layer 4 canary mismatch under investigation; "mismatch under investigation" public message published within 15 minutes; SOC has 15-minute window to add root-cause note before full public alert.
EC-DAT-01: Duplicate Layer 4 alert event (same hash mismatch reported twice from different workers); idempotent dedup; single alert record; no double public announcement.
EC-DAT-02: Same idempotency key with different alert body (race condition in alert generation); 422 idempotency-key-reuse; first alert wins; no silent override.
EC-DAT-11: Form 34A count mismatch on reproducibility check; Layer 4 critical alert; no suppression path; alert published immediately regardless of operator request.
EC-DAT-15: KEOG/ELOG PVT batch unavailable; Layer 6 degrades gracefully; "data unavailable" flag shown in Layer 6 panel; manual count and party-agent tally checks remain active.
EC-SEC-01: Attempt to suppress Layer 4 alert via API; no suppression endpoint exists; attempt returns 404 or 405 and is logged to audit trail.
EC-SEC-02: False Layer 6 PVT batch submitted with unrecognised station codes; station-code mapping validation rejects batch; tamper alert flagged to ECC analyst.
EC-SEC-05: Injection in alert description or analyst-resolution-note field; parameterised storage only; no dynamic query evaluation.
EC-SEC-09: Adversary studies detection algorithm to evade all 7 layers simultaneously; operationally infeasible due to independent signals, accountability chains, and human adjudication at Layer 7.
EC-SEC-16: Voter-level telemetry data or voter ID leaked to log lines; CI log-scanner fixture blocks deployment; only station-level aggregates permitted in telemetry logs.
EC-IDN-02: JWT expires mid ECC analyst session; in-flight alert resolution completes; next request returns 401 with silent refresh; no alert modification lost.
EC-IDN-10: SpiceDB down; fail closed on alert-write and resolution-write; public read-only endpoints (distributional, cryptographic, canary, report) continue; no layer marked clear.
EC-SCL-01: 55,393 simultaneous telemetry packets on election-day peak; worker pool auto-scales horizontally; Layer 2 SLO maintained; Layer 2 cycle <= 10 minutes confirmed under load test.
EC-SCL-04: Kafka consumer lag > 60 s on detection alert topic; page alert triggered; backpressure applied; no OOM on detection worker; no alert dropped.
EC-ML-01: Isolation Forest model drift on election day vs training baseline; drift alert raised via M28 model registry; analyst review ratio increases; T-30 recalibration planned and documented.
EC-ML-02: Layer 3 distributional test applied to station with < 25 votes; guard prevents test; station excluded from Layer 3 report with "insufficient count" annotation.
EC-ML-07: Alert flood from Layer 2 during early election-day turnout spike; dedup and ranked-priority queue applied; analyst inbox shows deduplicated summary; raw flood not surfaced.
EC-OPS-04: ECC analyst alert fatigue from Layer 2 false positives at large urban stations; snooze mechanism with mandatory reason code; T-30 recalibration loop incorporates snooze data.
EC-OPS-07: Central detection cluster partial failure during election day; Layer 1 kit-side alerts continue independently; Layers 2-6 each marked "layer degraded" not "clear"; IAB notified immediately.
EC-LEG-05: Layer 3 distributional result used as petition evidence in Supreme Court; methodology published; Benford caveat included; IAB statistician available as expert witness; Merkle proof of report timestamp presented.
EC-HUM-06: Screen readers on ECC console and IAB auditor dashboard; WCAG 2.2 AA; all seven alert workflow paths completable by analysts using assistive technology.

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
