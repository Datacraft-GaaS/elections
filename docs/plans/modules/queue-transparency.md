# M19 queue-transparency

| field | value |
|---|---|
| M-number | M19 |
| Home service | queue-svc (ADR-0020) |
| Owning work package | WP-4.2 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M19-queue-transparency.md |

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

Queue inputs:

FR-M19-001: Presiding Officer 30-min manual queue-length estimate via KIEMS kit prompt; signed by kit, timestamped, transmitted; legally authoritative input operating without any network dependency.
FR-M19-002: Kit authentication throughput per 15-min interval used as computed queue-length signal; smoothed with exponential moving average; derived as (cumulative-arrivals-estimate minus authenticated count).
FR-M19-003: Voluntary QR-code anonymous queue register at station entrance; token is UUID7 with no voter identity linkage; DPA 2019 lawful basis: no personal data collected.
FR-M19-004: Computer-vision anonymised headcount (optional signal); no image stored, no face identified, no data transmitted outside kit; count only, aggregated locally every 10 min; DPA 2019 s.26 public-interest basis confirmed in DPIA; visible signage mandatory.
FR-M19-005: Signal fusion: combine PO estimate, throughput rate, QR scans, and CV headcount into per-station snapshot every 5 min via lightweight regression and Kalman filter.

Derived metrics:

FR-M19-006: Compute estimated queue length, throughput rate (voters/min), estimated wait time (queue length divided by throughput), and estimated clear time (absolute EAT) per station per 5-min cycle.
FR-M19-007: Alert-level classification: green (<45 min wait), amber (45-120 min), red (>120 min), critical (>240 min); thresholds configurable per election cycle.
FR-M19-008: Confidence score per snapshot; snapshots with only one active signal source labelled "low confidence" on both public map and ECC panel.
FR-M19-009: Staleness flag when no new signal received for >10 min; stale snapshots distinctly labelled; last-seen timestamp displayed.

Public queue map:

FR-M19-010: Public queue map: all 55,393 stations colour-coded by alert level; compact GeoJSON endpoint; CDN-cached with 5-min TTL; updated every 5 min from latest data.
FR-M19-011: Per-station detail page: wait-time trend chart (by 30-min interval), last-updated timestamp, alert-level badge, active signal sources listed.
FR-M19-012: Public map p99 load time <=3 s under 1M concurrent users; >=95% requests served from CDN edge.
FR-M19-013: SMS opt-in: voter provides phone number via M13; receives SMS when home station drops below 30-min wait; p99 SMS delivery <60 s; phone stored hashed (SHA-256 + salt); deleted T+30 days.
FR-M19-014: DPA 2019 consent-basis SMS opt-in; consent recorded with timestamp; revocable at any time; processing stops immediately on revocation.

ECC escalation:

FR-M19-015: ECC alert at >90-min wait: dispatch alert to County Coordinator via M16; p99 delivery <2 min.
FR-M19-016: ECC alert at >180-min wait: roving engineer deployment recommendation delivered to County Coordinator via M16.
FR-M19-017: ECC alert at >240-min wait: escalate to the Commission Director; notify all accredited party agents for that station.
FR-M19-018: s.14A recommendation at >300-min wait: auto-generate pre-formatted recommendation document with supporting data hash; deliver to the Commission Chair within 5 min of trigger condition.
FR-M19-019: ECC LiveView channel (Phoenix LiveView): real-time queue status feed for ECC console; <10 s refresh; SSE endpoint for ECC console alongside.
FR-M19-020: All ECC alerts HSM-signed and appended to audit log; duplicate alerts deduplicated by idempotency key; no double-paging on repeated threshold breach.
FR-M19-021: s.14A recommendation document: the Commission Chair electronic HSM-signature workflow; recommendation logged separately from the Chair decision; advisory-only status clearly marked.

Disenfranchisement and attendance:

FR-M19-022: Queue-not-cleared event: if PO estimate shows >0 queue at station close and station closed before all queued voters voted, log DisenfranchisementFlag with flag_type=queue_not_cleared.
FR-M19-023: Disenfranchisement flag routed to IAB (M15) and M23 grievance channel within 60 s of detection.
FR-M19-024: Attendance counter 1: PO mechanical clicker tally logged every 30 min as part of FR-M19-001 prompt (Form 32B sync field).
FR-M19-025: Attendance counter 2: CV anonymised headcount logged locally every 10 min; no image or face data transmitted; DPA 2019 DPIA covers this counter.
FR-M19-026: Attendance counter 3: QR-code voluntary roster count aggregated in real time; no voter identity linked to token.
FR-M19-027: Reconciliation check: authentications > entries x 1.02 triggers result-hold flag; entries > authentications x 1.5 triggers systemic-disenfranchisement investigation flag.
FR-M19-028: Citizen or party-agent queue report via USSD/WA; rate-limited 50/hr per IP; incorporated into next snapshot cycle via signal fusion.
FR-M19-029: Rate-limiting keyed by IP+/64 and session; anti-spam protection; burst allowed then shed; no bypass via header manipulation.

Offline and resilience:

FR-M19-030: All three queue inputs operate without connectivity; kit queues estimates, headcounts, and QR logs locally; syncs on reconnect; ECC receives historical snapshots on reconnection with gap audit.
FR-M19-031: Configurable signal weights and alert thresholds per election cycle; changes require dual-officer authorisation and audit log entry.

Post-election analytics:

FR-M19-032: Historical snapshot archive: all snapshots, attendance counts, alert events, and s.14A triggers retained permanently for post-election analysis.
FR-M19-033: Post-election open dataset published within T+30 days: per-station wait time by 30-min interval, attendance counts, alert events, disenfranchisement events, in CSV + JSON.
FR-M19-034: Post-election analytics report: peak hours by county, turnout correlation, disenfranchisement event map; delivered to Parliament and Auditor-General.

## Acceptance criteria

AC-M19-01: Snapshot computed and published within 5 min of simulated check-in spike across 55,393 concurrent station streams (load test).
AC-M19-02: DisenfranchisementFlag delivered to M16 ECC p99 <2 min from threshold breach (integration test; NFR-19.02).
AC-M19-03: s.14A recommendation generated and delivered to the Commission Chair within 5 min of >300-min trigger condition (integration test).
AC-M19-04: Citizen USSD queue report incorporated into the next snapshot cycle (integration test).
AC-M19-05: Public map loads p99 <=3 s at 1M concurrent users; >=95% served from CDN edge (load test; NFR-19.03).
AC-M19-06: Snapshot labelled "low confidence" when only one signal source is active (unit test; all three source-missing combinations tested).
AC-M19-07: Staleness flag raised when no new signal received for >10 min; cleared immediately on next signal arrival (unit test; boundary at exactly 10 min).
AC-M19-08: SMS opt-in phone number stored hashed only; not retrievable in plain text; deleted at T+30 days (data-retention audit; SHA-256 + salt verified).
AC-M19-09: CV headcount stores no images and transmits no face data; confirmed by independent DPA privacy review before production (privacy audit).
AC-M19-10: Attendance reconciliation flag fires when authentications > entries x 1.02 (unit test; boundary table at 1.02x and 1.019x).
AC-M19-11: All four ECC escalation tiers (90, 180, 240, 300 min) generate correctly scoped alerts to the correct recipients with no false escalations (integration test).
AC-M19-12: Post-election open dataset published within T+30 days of election day (process audit; publication timestamp verified).
AC-M19-13: 99.5% availability during election day (routine SLO per appendix M19 entry); burn-rate alert fires before budget exhausted (SLO measurement).

## Applicable edge cases

EC-CON-01: Kit offline for 7+ days; queue estimates, headcounts, and QR logs accumulate in local ring-buffer queue; signed and sequenced; no loss on reconnect; ECC receives historical snapshots with gap audit.
EC-CON-02: 3G (500 kbps, 300 ms RTT): CBOR+zstd GeoJSON snapshot payload negotiated; public map meets p99 3 s; M13 queue tab tested on throttled emulator.
EC-CON-04: Site partition between primary and BVI mirror; last-known snapshot retained; stale label shown on map; circuit breaker opens; ECC notified within 60 s.
EC-CON-05: Flapping BVI link; backoff+jitter on snapshot ingest; duplicate BVI events deduplicated; no phantom queue spike generated.
EC-TIM-04: DST/TZ on diaspora kits: all snapshot timestamps stored UTC; EAT displayed; queue clear-time computed in UTC and converted for display.
EC-TIM-07: s.14A close time moves (authorised extension from 17:00 to 19:00 under Elections Act s.14A); queue model recomputes estimated clearance against new close time within one 5-min snapshot cycle; HSM-signed extension; forged extension rejected.
EC-DAT-01: Duplicate citizen queue report same idempotency key and body; cached response returned; fusion not double-counted; no re-execution.
EC-DAT-02: Same citizen report idempotency key, different body; rejected with 422 idempotency-key-reuse; not fused into snapshot.
EC-DAT-05: Conflicting signals (CV says 10 voters, USSD says 200 voters); fusion model confidence drops; snapshot labelled "uncertain"; no crash; both raw values logged for post-election analysis.
EC-DAT-09: Unicode station names in 11 locales; NFC normalisation applied; ICU collation; no truncation on public map or per-station detail page.
EC-DAT-10: Zero queue length is a valid derived value; no divide-by-zero in wait-time computation; public map renders "0 min estimated wait".
EC-IDN-02: JWT expires during citizen report submission; in-flight request completes; next request returns 401 with silent-refresh hint.
EC-IDN-10: SpiceDB down; fail closed on accredited ECC flag-feed writes; public map read-only CDN tier continues; class-C reads allowed for operators.
EC-SEC-05: Injection on queue report text and USSD input fields; parameterised queries only; Semgrep compile-time and ZAP dynamic scan required.
EC-SEC-09: Rate-limit citizen queue reports; keyed by IP+/64; anti-spam; burst allowed then shed; no bypass via X-Forwarded-For or similar header manipulation.
EC-SEC-16: Secrets or PII in queue report free-text refused at ingest; telemetry scrubbed; log-scanner fixture in CI.
EC-ML-01: Signal fusion model drift due to new station layouts or behavioural changes; accuracy monitor triggers recalibration alert; ECC notified; model card updated before next election cycle.
EC-ML-07: Alert flood when 1,000+ stations breach threshold simultaneously; dedup plus rate-limit plus severity ranking applied; ECC inbox protected from saturation.
EC-SCL-01: 1M concurrent public map users; >=95% requests served from CDN edge; origin p99 <=500 ms (k6 + Gatling 5 regions).
EC-SCL-03: Hot-partition on single station with extremely high event rate (e.g., viral social-media attention); stream-id spread across partitions; consumer lag alert fires.
EC-SCL-04: Consumer lag >60 s on eel.queue.snapshot.v1 topic; backpressure activated; page fired; no OOM condition.
EC-LEG-03: s.44A invoked at station; dual-officer S44AEvent generated; queue snapshot at closure time included in evidence package; DisenfranchisementFlag auto-generated.
EC-LEG-08: s.14A evidence-request at T+6d 23 h; historical snapshot dataset not freeze-blocked; 200 concurrent bundle requests served within 30 s each.
EC-HUM-06: Screen readers: public queue map and per-station detail page fully navigable; WCAG 2.2 AA verified with axe-core and manual NVDA test.
EC-HUM-08: 11 languages: station names, wait-time labels, alert banners, and SMS text localised; no truncation on long-string locales; 150% pseudo-locale tested in CI.
EC-OPS-08: CV camera fails mid-election-day; signal quarantined; fusion continues with remaining three signals; ECC notified; no snapshot gap larger than one 5-min cycle.

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

- CV camera integration: the computer vision headcount source is marked optional and requires the Commission procurement sign-off on camera hardware at each station; design treats it as an optional signal and the module operates without it.
- Signal fusion weights: the relative weighting of USSD reports vs check-in rate vs CV is currently equal; the Commission data team needs to calibrate on historical election data before G3.
