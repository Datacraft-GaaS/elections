# M60 medical-first-response

| field | value |
|---|---|
| M-number | M60 |
| Home service | training-svc (ADR-0020) |
| Owning work package | WP-5.4 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M60-medical-first-response.md |

## Purpose

Manages medical incident reporting, dispatch requests, and cluster alerts for election-day medical emergencies at polling stations, integrating with KRCS, MoH, and NDMU partners, with strict no-PHI schema enforcement.

## Surfaces and languages

| surface | technology | path under services/training-svc/ |
|---|---|---|
| Medical incident admin | Django | django/medical/ |
| Mobile incident reporting (M31 integration) | Flutter (via M31 poll-clerk app) | apps/poll-clerk (field in M31) |
| Internal API | Rust (axum) | crates/training-api/ (medical endpoints) |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| MedicalIncident | id, station_id, incident_type (injury/illness/cardiac/other), severity (minor/moderate/critical), reported_at, reporter_id, status (reported/dispatched/resolved/escalated) | C | T1-consensus |
| DispatchRequest | id, incident_id, dispatched_to (KRCS/MoH/NDMU), dispatched_at, eta_minutes, acknowledged_at | C | T1-consensus |
| ClusterAlert | id, county_code, incident_count, alert_threshold, raised_at, resolved_at | C | T1-consensus |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/medical/incidents | 3 accredited | JWT+SpiceDB | yes | report medical incident |
| GET /v1/medical/incidents | 3 accredited | JWT+SpiceDB | n/a | list incidents (county-scoped) |
| POST /v1/medical/incidents/{id}/dispatch | 3 accredited | JWT+SpiceDB | yes | request dispatch |
| PUT /v1/medical/incidents/{id}/resolve | 3 accredited | JWT+SpiceDB | yes | mark resolved |
| GET /v1/medical/cluster-alerts | 3 accredited | JWT+SpiceDB | n/a | active cluster alerts |
| POST /v1/medical/cluster-alerts/{id}/resolve | 3 accredited | JWT+SpiceDB | yes | resolve cluster alert |

OpenAPI: packages/openapi-contracts/training/medical-first-response.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.training.medical_incident_reported.v1 | emits | eel.training.medical.v1 | operational | no | at-least-once idempotent |
| eel.training.medical_cluster_alert.v1 | emits | eel.training.medical.v1 | operational | no | at-least-once idempotent |
| eel.training.dispatch_requested.v1 | emits | eel.training.medical.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| KRCS | krcs (tier B double) | integrations/ | yes | circuit breaker; dispatch alert to ECC |
| MoH | moh (tier B double) | integrations/ | yes | circuit breaker; dispatch alert to ECC |
| NDMU | ndmu (tier B double) | integrations/ | yes | circuit breaker; dispatch alert to ECC |
| M16 election-command-centre | internal | crates/training-api/ | yes | advisory; cluster alert forwarded |
| M31 mobile-poll-clerk | internal | crates/training-api/ | yes | hard dependency; incident reporting surface |

## ML and AI

none

## Dependencies

- Modules: M16 (ECC cluster alert), M31 (mobile incident surface)
- Platform: P01 (SpiceDB), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py

## Feature checklist

Pre-election setup:

1. FR-M60-001: Register pre-positioned first-aid teams and ambulance units by county before T-2 (FR-M60-01).
2. FR-M60-002: Capture unit attributes: unit ID, type (first_aid_team, ambulance, field_hospital), county, sub-county, GPS coordinates, contact number, capacity.
3. FR-M60-003: Lock KRCS unit registration at T-2; read-only access for field coordinators after lock.
4. FR-M60-004: Track pre-election coordination calendar with MoH, NDMU, and KRCS: tabletop exercises, pre-position confirmation, and protocol rehearsals (FR-M60-08).
5. FR-M60-005: Record minutes and action items from each coordination calendar event.

Incident reporting:

6. FR-M60-006: Report medical incident from M31 poll-clerk mobile app or M60 web interface (FR-M60-02).
7. FR-M60-007: Classify incident by type: cardiac, trauma, heat_exhaustion, crowd_crush, respiratory, other.
8. FR-M60-008: Record severity: minor, moderate, critical.
9. FR-M60-009: Record persons_affected as aggregate count only; no patient names, IDs, or diagnoses in any field (FR-M60-02).
10. FR-M60-010: Auto-prefill polling station code from field officer login session.
11. FR-M60-011: Enforce no-PHI schema: reject any field containing personal health identifiers; Semgrep rule and fixture test confirm (FR-M60-14).
12. FR-M60-012: Submit incident offline when network unavailable; queue on M31; sync within 2 minutes of reconnection.

Dispatch coordination:

13. FR-M60-013: Create dispatch request for any moderate or critical incident; request visible to county KRCS Dispatch Coordinator within 15 s p99 (FR-M60-03).
14. FR-M60-014: Allow KRCS Dispatch Coordinator to assign nearest unit and enter ETA in minutes.
15. FR-M60-015: Display ETA to the Commission Field Medical Liaison in real time.
16. FR-M60-016: Record dispatch status transitions: dispatch_requested, dispatch_en_route, on_scene, resolved with timestamps.
17. FR-M60-017: Dispatch request idempotent; double-fire safe; exactly one dispatch record created.
18. FR-M60-018: Alert operations when critical incident has no dispatch within 5 minutes.
19. FR-M60-019: Degrade gracefully when KRCS dispatch system is offline; circuit breaker opens; ECC alerted on persistent failure.

NDMU escalation:

20. FR-M60-020: Auto-generate escalation notice to NDMU Liaison for critical incidents or cluster events: 3+ moderate incidents in one sub-county within 2 hours (FR-M60-04).
21. FR-M60-021: Record NDMU Liaison acknowledgement and response status update.
22. FR-M60-022: Re-escalate to Commission Director of Operations and M16 ECC if NDMU escalation unacknowledged after 30 minutes.
23. FR-M60-023: Cluster trigger idempotent; escalation idempotent; no duplicate notices on rapid repeat submission.

Epidemic surveillance:

24. FR-M60-024: Forward aggregate incident counts by type per county to MoH surveillance at 2-hour intervals during election window T-1 to T+1 (FR-M60-05).
25. FR-M60-025: Signal includes county code, incident-type breakdown, total persons affected, timestamp; no individual health records forwarded.
26. FR-M60-026: Queue MoH signals on failure; retry every 15 minutes; queue up to 6 hours; SRE alert if queue exceeds threshold.
27. FR-M60-027: Record MoH acknowledgement per signal.

ECC integration:

28. FR-M60-028: Push all open incidents to M16 ECC via SSE; incident count per station and county; severity colour-coded; no individual voter or officer data (FR-M60-06).
29. FR-M60-029: ECC SSE feed incident latency p99 < 30 s.

Reporting and admin:

30. FR-M60-030: Generate post-election medical summary report at T+1: incidents by type and severity per county, KRCS dispatch count, average ETA, unresolved incidents, MoH signals sent (FR-M60-07).
31. FR-M60-031: Admin list of incidents with filter by county, severity, status.
32. FR-M60-032: County medical dashboard with incident heatmap, severity colour layer, open vs resolved count, cluster alert banner.
33. FR-M60-033: Audit trail of all incident and dispatch state transitions in P05.
34. FR-M60-034: County-scoped access; no cross-county incident visibility without national role; SpiceDB policy enforced.
35. FR-M60-035: Free-text incident notes scanned for PII patterns before storage; field stripped with alert to submitter.

## Acceptance criteria

1. AC-M60-01: PHI schema enforcement: any field containing patient name, national ID, or diagnosis rejected at API boundary with 422; Semgrep rule detects patterns at CI; fixture test confirms rejection.
2. AC-M60-02: Dispatch request visible to KRCS Dispatch Coordinator p99 < 15 s from incident submission; integration test with KRCS double under 10,000 concurrent submission load.
3. AC-M60-03: Critical incident not dispatched within 5 minutes triggers alert to M16 ECC; integration test with M16 double.
4. AC-M60-04: NDMU escalation sent within 5 minutes of cluster trigger (3 moderate incidents, same sub-county, 2-hour window); property test with synthetic incident stream.
5. AC-M60-05: NDMU re-escalation sent to Director of Operations within 30 minutes of unacknowledged first escalation; timed integration test.
6. AC-M60-06: Cluster alert idempotent; no duplicate NDMU escalation on rapid repeat incident submission; property test.
7. AC-M60-07: ECC SSE feed incident latency p99 < 30 s; measured under 10,000 concurrent submission load via M16 SSE subscriber.
8. AC-M60-08: MoH 2-hourly signals delivered on schedule during election window; 100% delivery rate in staging election-window simulation.
9. AC-M60-09: Incident system availability T-3 to T+1 >= 99.5%; SLO burn-rate alert fires before threshold breach.
10. AC-M60-10: Offline M31 incident submission syncs within 2 minutes of reconnection; zero events lost; network-partition integration test.
11. AC-M60-11: Dispatch request idempotent; double-fire integration test; exactly one dispatch record created per incident.
12. AC-M60-12: Post-election medical summary report generated and accessible by T+2; Playwright e2e confirms report populated for all counties with correct counts.
13. AC-M60-13: County-scoped SpiceDB policy blocks cross-county incident read without national role; access-control integration test.

## Applicable edge cases

EC-DAT-01: duplicate incident report with same idempotency key reuses cached response; no second incident created.
EC-DAT-02: same idempotency key with different incident body returns 422 idempotency-key-reuse; incident rejected.
EC-DAT-05: poison incident report (persons_affected implausibly large or negative); DLQ with validation reason; partition continues processing.
EC-DAT-09: Unicode characters in incident notes (Swahili or other locale text); NFC normalised; grapheme-safe truncation applied.
EC-DAT-10: zero incidents in a county during entire election day; cluster alert not raised; no division errors in summary report.
EC-DAT-14: oversized incident description field exceeds 1 MB; outbox rejects with 422; submitter notified.
EC-SEC-05: injection on incident description and notes fields; parameterised queries only; ZAP active scan and Semgrep at CI.
EC-SEC-06: SSRF via external MoH or NDMU webhook URL fields; allowlist enforced; private IP ranges blocked; DNS rebinding guard.
EC-SEC-09: rate-limit evasion on incident submission endpoint; key by subject + device + /64; 429 after burst; priority shed active.
EC-SEC-16: PHI must not appear in logs or traces; log-scanner fixture blocks Tier A/B health identifiers in telemetry; CI regex.
EC-IDN-02: JWT expires mid-incident submission; in-flight request completes; next request returns 401 and triggers silent refresh.
EC-IDN-10: SpiceDB down; writes fail closed; county-scoped reads from read-replica only; 503 problem+json returned.
EC-SCL-04: Kafka consumer lag > 60 s on medical incident topic; SRE paged; backpressure applied; no OOM.
EC-SCL-07: 250k simultaneous incident submissions at election-morning peak; SLAs met at 1x load; <= 10% p99 regression at 5x load in staging simulation.
EC-CON-05: flapping KRCS / MoH / NDMU external APIs; backoff-with-jitter; retry budget consumed; M16 ECC alerted on persistent failure.
EC-CON-02: 3G (500 kbps, 300 ms RTT) from M31 field device; incident submission p99 < 1 s; CBOR+zstd negotiated.
EC-MOB-01: OS kills M31 mid-incident submission; resumable offline queue; no duplicate event created on reconnect.
EC-MOB-02: M31 storage full during offline incident queue accumulation; graceful refusal with bounded queue alert to field officer.
EC-TIM-04: field officer device clock in non-EAT timezone; all incident timestamps stored in UTC; EAT displayed in dashboard.
EC-IDN-05: rooted or Frida-instrumented device attempts incident submission; RASP terminates session; tamper event logged.
EC-HUM-05: incident involving assisted voting (accessibility need); incident type captures the operational need without any PHI.
EC-HUM-06: field officer uses screen reader on M60 web interface; full task completion at WCAG 2.2 AA.
EC-LEG-03: s.44A invoked at polling station where medical incident is active; medical incident continues independently; no cross-contamination of records.
EC-OPS-07: the Commission Field Medical Liaison no-show; county coordinator assumes liaison role; credential reassignment exactly-once; old liaison access revoked.
EC-OPS-08: disk full on training-svc node; bounded incident queue; 507 returned; SRE alert; no data corruption.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Incident FSM, cluster alert threshold, no-PHI guard | nextest | M60-UNIT- |
| property | Cluster threshold boundary; idempotency | proptest | M60-PROP- |
| contract | KRCS/MoH/NDMU doubles; M16 ECC alert; M31 mobile | Pact | M60-PACT- |
| integration | Full incident-dispatch-resolve lifecycle | nextest + testcontainers | M60-INT- |
| e2e | Admin dashboard; dispatch workflow | Playwright | M60-E2E- |
| security | Semgrep PHI rules; ZAP; log scanner | ZAP + Semgrep | M60-SEC- |

Tags: `#[trace("FR-M60-003","AC-M60-01","EC-SEC-16")]`

## Runbooks and dashboards

- docs/runbooks/training-svc-degradation.md (extend with medical section)
- docs/runbooks/medical-cluster-alert-response.md (new)
- platform/observability/training-svc/medical-dashboard.json
- platform/observability/training-svc/slo-alerts.yaml

## Open questions

- MoU status with KRCS, MoH, and NDMU for API integration; doubles gate development until MoUs signed.
- Cluster alert threshold values (incident count and time window) subject to operational policy from medical coordination committee.
