# M16 election-command-centre

| field | value |
|---|---|
| M-number | M16 |
| Home service | ecc-svc (ADR-0020) |
| Owning work package | WP-4.1 (docs/plans/05-wave-4-operations.md) |
| Size | XL |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M16-election-command-centre.md |

## Purpose

The National Intelligence and Situational Awareness (NIS) loop for the Commission leadership and operations teams: aggregate telemetry from all kit streams, results, anomalies, grievances, speech flags, queue data, and inventory into a unified command view, enabling real-time dispatch decisions, s.44A authorisations, and coordinated incident response.

## Surfaces and languages

| surface | technology | path under services/ecc-svc/ |
|---|---|---|
| ECC multi-role dashboard | Elixir (Phoenix LiveView) | ex/ecc-live/ |
| Mobile ECC app (field command) | Flutter on packages/kit-sdk-flutter | apps/ecc-mobile/ |
| ECC API | Rust (axum + utoipa) | crates/ecc-api/ |
| Alert + dispatch logic | Rust | crates/ecc-domain/ |
| Incident report generator | Rust + Typst | crates/ecc-reports/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| KitTelemetrySnapshot | snap_id, kit_serial, station_code, battery, connectivity, gps_fix, temp, last_seen, error_codes | C | T2 |
| ECCAlert | alert_id, source_module, severity, category, description, status, assigned_to, resolved_at | C | T2 |
| S44AEvent | event_id, station_code, authorised_by, authorised_at, reason, dual_officer, sig | C | T1-audit |
| Dispatch | dispatch_id, alert_id, personnel_ids, instruction, status, acked_at | C | T2 |
| IncidentReport | report_id, election_id, category, content_hash, sig, published_at | D | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/ecc/command.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /ecc/v1/summary | 3 accredited | JWT+SpiceDB | n/a | Live national summary: kits, alerts, results progress |
| GET /ecc/v1/kits | 3 accredited | JWT+SpiceDB | n/a | Kit telemetry heatmap |
| GET /ecc/v1/alerts | 3 accredited | JWT+SpiceDB | n/a | Alert feed, filterable by severity/module/area |
| POST /ecc/v1/alerts/{id}/dispatch | 3 accredited | JWT+SpiceDB | yes | Dispatch personnel to alert |
| PATCH /ecc/v1/alerts/{id} | 3 accredited | JWT+SpiceDB | yes | Update alert status |
| POST /ecc/v1/s44a | 3 accredited | JWT+SpiceDB | yes | Record s.44A authorisation (dual-officer) |
| GET /ecc/v1/anomalies | 3 accredited | JWT+SpiceDB | n/a | Anomaly/fraud flags from M17/M26 |
| GET /ecc/v1/queue | 3 accredited | JWT+SpiceDB | n/a | Queue status from M19 |
| POST /ecc/v1/reports | 3 accredited | JWT+SpiceDB | yes | Publish incident report |
| GET /ecc/v1/inventory | 3 accredited | JWT+SpiceDB | n/a | Inventory status from M20 |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.ecc.alert_created.v1 | emits | eel.ecc.alert.v1 | operational | yes | at-least-once idempotent |
| eel.ecc.alert_resolved.v1 | emits | eel.ecc.alert.v1 | operational | yes | at-least-once idempotent |
| eel.ecc.s44a_authorised.v1 | emits | eel.ecc.s44a.v1 | operational | yes (dual) | exactly-once |
| eel.ecc.dispatch_created.v1 | emits | eel.ecc.dispatch.v1 | operational | yes | at-least-once idempotent |
| eel.bvi.session_accepted.v1 | consumes | eel.bvi.session.v1 | operational | yes | at-least-once idempotent |
| eel.bvi.distress_received.v1 | consumes | eel.bvi.security.v1 | operational | yes | exactly-once |
| eel.rts.form_ingested.v1 | consumes | eel.rts.form.v1 | event-source | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| BVI back-end M06 | internal | integrations/bvi-backend/ | yes | telemetry feed; circuit breaker |
| RTS back-end M10 | internal | integrations/rts-backend/ | yes | form ingest status feed |
| Verifier app M13 | internal | integrations/verifier/ | yes | grievance + anomaly relay |
| Speech integrity M17 | internal | integrations/speech/ | yes | content flags feed |
| Queue transparency M19 | internal | integrations/queue/ | yes | disenfranchisement flags |
| Inventory M20 | internal | integrations/inventory/ | yes | kit/seal status |
| Anomaly/fraud M26 | internal | integrations/anomaly/ | yes | fraud alert feed |
| Personnel dispatch M33 | internal | integrations/dispatch/ | yes | personnel assignment |
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed on s44A + report sign |

## ML and AI

No ML runs inside this module. It consumes flags from M17 (speech integrity) and M26 (anomaly/fraud) and presents them in the alert feed.

## Dependencies

- Modules: M08, M10, M13, M17, M18, M19, M20, M26, M33
- Platform: P05 (audit), P06 (HSM)
- Packages: eel-crypto-rs, eel-outbox-rs, kit-sdk-flutter

## Feature checklist

1. FR-M16-001: national live summary dashboard: kits online/offline, forms received, tokens issued, alerts by severity, across all levels.
2. FR-M16-002: kit telemetry heatmap: constituency/ward/station; filter by battery, connectivity, GPS, error state.
3. FR-M16-003: real-time alert feed: severity-ranked; categories include anomaly, grievance, distress, queue, speech, inventory.
4. FR-M16-004: alert dispatch: assign personnel (M33) to alert; track acknowledgement and resolution.
5. FR-M16-005: s.44A authorisation flow: dual-officer; HSM-signed; reasons logged; propagated to M08/M06.
6. FR-M16-006: distress signal from kit (M06) escalated to ECC alert within 5 s.
7. FR-M16-007: speech integrity flags from M17 displayed with severity and evidence link.
8. FR-M16-008: queue transparency flags from M19 displayed with disenfranchisement risk rating.
9. FR-M16-009: inventory alerts from M20: missing kit, broken seal, low reserves.
10. FR-M16-010: incident report generator: structured report, HSM-signed, anchored to tlog.
11. FR-M16-011: role-based dashboards: national director, county officers, ROs, legal desk, media desk, SOC; no cross-scope visibility.
12. FR-M16-012: NIS loop SLA: all source feeds aggregated and alerts created within 30 s of upstream event.
13. FR-M16-013: mobile ECC companion app (Flutter): field supervisors receive alerts; dispatch offline-resilient.
14. FR-M16-014: full audit trail: every command action signed, timestamped (HLC), and anchored to immutable log.
15. FR-M16-015: RBAC enforced via SpiceDB; at least 8 distinct roles; role matrix documented.
16. FR-M16-016: results ingestion dashboard: per-station, per-ward, per-constituency tallies with latency indicators.
17. FR-M16-017: results publication control: dual-sign required before public portal update; quorum enforced.
18. FR-M16-018: HSM signing for all publication events; quorum policy configurable per election type.
19. FR-M16-019: alert management: station-closed, dispute-raised, tampering-detected; routed to correct officer role.
20. FR-M16-020: incident lifecycle: create, assign, escalate, resolve with full audit trail and timestamps.
21. FR-M16-021: resource allocation: polling staff, ballot paper counts, emergency deployments tracked per station.
22. FR-M16-022: internal broadcast: send message to all ROs and county officers; delivery receipt tracked.
23. FR-M16-023: escalation templates: pre-approved wording for alert escalation; legal review gate.
24. FR-M16-024: legal/dispute workflow: flagging, court referral, injunction management with status tracking.
25. FR-M16-025: media briefing preparation: auto-generated summary reports from live data.
26. FR-M16-026: system health monitoring: integration with M15 IAB dashboard feeds; SLO burn-rate view.
27. FR-M16-027: SLO burn-rate view: per-module alerts fed from Prometheus/Grafana; threshold configurable.
28. FR-M16-028: degradation-mode control: escalated to national director only; dual-sign required; immutable log.
29. FR-M16-029: WCAG 2.2 AA compliance for all web surfaces; automated checks in CI.
30. FR-M16-030: multilingual support: English and Kiswahili on web; 11 languages on mobile companion.
31. FR-M16-031: p95 dashboard load time < 500 ms under election-day concurrency.
32. FR-M16-032: alert delivery guaranteed within 10 s from originating event to officer UI.
33. FR-M16-033: 99.99% availability target on election day; measured by uptime probe every 30 s.
34. FR-M16-034: alert deduplication: same source key within 60 s window collapses to single alert with count.
35. FR-M16-035: alert storm throttle: when > 1000 alerts/min, rate-limit to DLQ and send aggregated digest to ECC.
36. FR-M16-036: alert auto-close: unacknowledged low-severity alerts auto-close after 2 h with reason logged.
37. FR-M16-037: officer on-call rotation: on-call list per role; alerts routed to active on-call officer first.
38. FR-M16-038: escalation timer: unacknowledged critical alert escalates to next tier after configurable timeout.
39. FR-M16-039: WebSocket push to connected ECC sessions on every new alert; no polling required.
40. FR-M16-040: REST API for all ECC operations; versioned at /api/v1; OpenAPI spec published.
41. FR-M16-041: API rate limiting: per-role quotas; excess returns 429 with Retry-After header.
42. FR-M16-042: API authentication via short-lived JWTs signed by Keycloak; refresh token rotation enforced.
43. FR-M16-043: API audit log: every mutating request logged with actor, resource, action, timestamp, IP.
44. FR-M16-044: mobile offline mode: alerts cached locally; dispatch actions queued; sync on reconnect.
45. FR-M16-045: mobile push notifications: FCM/APNs; fallback SMS for critical alerts when app is closed.
46. FR-M16-046: mobile biometric unlock: fingerprint/face-ID enforced before viewing sensitive data.
47. FR-M16-047: mobile session timeout: auto-lock after 5 min inactivity; re-auth required.
48. FR-M16-048: mobile GPS location stamp on all dispatch actions; consent captured at session start.
49. FR-M16-049: tally ingestion pipeline: accepts signed forms from M08; validates signature before display.
50. FR-M16-050: tally anomaly flag: automatic if submitted tally deviates > 5% from projections; manual review gate.
51. FR-M16-051: station status board: real-time open/closed/contested state per polling station.
52. FR-M16-052: spoilt ballot tracker: per station counts; aggregate by ward, constituency, county.
53. FR-M16-053: rejected form tracker: reason codes; escalation path to RO then national command.
54. FR-M16-054: announcement scheduler: pre-draft announcements with publish-at time; dual-approve gate.
55. FR-M16-055: press query queue: media desk receives structured questions; command centre drafts responses.
56. FR-M16-056: injunction tracker: active injunctions with affected station list; auto-suppresses tally display.
57. FR-M16-057: court order integration: court order number linked to injunction; PDF stored in MinIO.
58. FR-M16-058: candidate petition tracker: petition received, status, hearing date, outcome.
59. FR-M16-059: recount authorisation: dual-sign; reasons logged; affected station locked pending recount.
60. FR-M16-060: manual override audit: every manual correction requires second officer confirmation and reason.
61. FR-M16-061: data export: CSV/PDF of tallies, incidents, alerts for legal and archival use; signed.
62. FR-M16-062: retention policy: election-day data retained 7 years; archival after 90 days to cold storage.
63. FR-M16-063: role delegation: national director can temporarily delegate role to deputy; logged and time-limited.
64. FR-M16-064: session concurrency limit: maximum 3 concurrent sessions per officer; oldest terminated on 4th.
65. FR-M16-065: suspicious login alert: new device or unusual IP triggers MFA re-challenge and SOC notification.
66. FR-M16-066: SOC dashboard: all security events, failed auth attempts, anomalous API calls in real time.
67. FR-M16-067: canary token deployment: honeypot data in restricted views; access triggers SOC alert.
68. FR-M16-068: secrets scanning: alert descriptions and broadcast content rejected if they match secret patterns.
69. FR-M16-069: configuration change audit: every system config change requires dual-approve and is signed.
70. FR-M16-070: health check endpoint: /healthz returns component status; used by Prometheus scrape.
71. FR-M16-071: chaos drill mode: simulate feed loss, HSM failure, SpiceDB outage; records drill outcome.
72. FR-M16-072: performance budget enforcement: CI fails if p95 API response > 500 ms on load test.
73. FR-M16-073: dark mode support: all web surfaces; respects OS preference; persists per user.
74. FR-M16-074: keyboard navigation: all interactive elements reachable via keyboard; focus indicators visible.
75. FR-M16-075: screen reader compatibility: ARIA labels on all dynamic alert cards and status indicators.

## Acceptance criteria

AC-M16-01: NIS loop latency <= 30 s from upstream event to alert; verified by integration test under election-day load.
AC-M16-02: distress alert appears in ECC within 5 s of M06 distress event; verified by chaos exercise with SoftHSM.
AC-M16-03: s.44A flow requires dual-officer HSM auth on physically separate sessions; single officer cannot complete; verified by UAT.
AC-M16-04: RBAC: county officer cannot view national alerts and national director cannot view county-restricted data from wrong county; verified by automated security tests.
AC-M16-05: incident report is HSM-signed and anchored to tlog before any external publication; verified by integration test.
AC-M16-06: mobile companion app queues dispatch actions offline and syncs correctly on reconnect without data loss; verified by chaos test.
AC-M16-07: 99.99% availability measured over election day window via 30 s uptime probe; SLO burn-rate alert fires before budget exhausted.
AC-M16-08: p95 dashboard load time < 500 ms under simulated election-day concurrency of 500 concurrent officers; verified by k6 load test.
AC-M16-09: alert delivery end-to-end <= 10 s from originating event to officer UI WebSocket push; verified by integration timing assertion.
AC-M16-10: results publication blocked if fewer than two authorised signatories have signed; HSM quorum check enforced; verified by contract test.
AC-M16-11: all web surfaces pass WCAG 2.2 AA automated scan (axe-core) with zero critical violations in CI.
AC-M16-12: secrets in alert descriptions or broadcast content are rejected at API boundary with HTTP 422; verified by unit test.
AC-M16-13: degradation-mode control requires dual-sign from national director level; lower roles cannot activate; verified by RBAC security test.
AC-M16-14: every mutating ECC API action is present in audit log with actor, resource, action, and HLC timestamp within 1 s of execution; verified by integration test.

## Applicable edge cases

EC-DAT-01: duplicate alert arrives with same station-alert key within 60 s; system deduplicates to single alert with count incremented, no duplicate notification sent.
EC-DAT-02: partial tally form received with missing ward fields; form rejected with structured error, logged, RO notified; no partial tally written to live view.
EC-SEC-05: officer attempts to access another county's restricted alerts via direct URL; SpiceDB denies; 403 returned; SOC event emitted.
EC-SEC-16: alert description contains a secret-pattern string (e.g. API key fragment); content rejected at ingestion boundary; originator receives 422.
EC-IDN-02: officer session token expired during active incident management; re-auth prompt shown; in-flight form preserved for 5 min; no data lost.
EC-IDN-10: SpiceDB becomes unavailable; all write operations fail closed; read-only cached RBAC used for view operations; SOC alerted immediately.
EC-SCL-04: single Kafka partition receives > 5000 alert events/s; consumer lag monitored; auto-scale consumer group triggered; p95 latency maintained.
EC-IDN-09: s.44A dual-officer flow: second officer session is on same device as first; system detects shared session fingerprint and blocks commit; both must re-auth on distinct sessions.
EC-OPS-01: canary deployment of ECC service; 5% of traffic routed to new version; alert latency metrics compared; rollback triggered if p95 regresses > 20%.
EC-OPS-02: rolling restart of ECC pods during election day; WebSocket connections re-established within 3 s; no alert lost via durable subscription.
EC-OPS-03: Prometheus scrape target goes stale; Grafana shows "No Data" panel; SLO burn-rate alert fires within 2 min; on-call paged.
EC-OPS-04: MinIO court-order bucket unreachable; injunction PDF upload queued in local buffer; retry with exponential back-off; operator alerted after 3 failures.
EC-OPS-05: database primary failover during peak results ingestion; application retries on replica within 5 s; in-flight tally write replayed from WAL; no tally duplicated.
EC-OPS-06: TLS certificate expiry within 7 days; automated renewal attempted; if renewal fails, alert escalated to SOC and ops team with 48 h lead time.
EC-OPS-07: config change deployed without dual-approve gate bypassed (e.g. direct kubectl apply); canary token in config triggers SOC alert; change rolled back.
EC-OPS-08: log ingestion pipeline to Loki saturates; local buffer fills; oldest logs dropped with count metric incremented; operator alerted.
EC-OPS-09: Grafana dashboard unavailable; ECC falls back to raw Prometheus query endpoint for SLO burn-rate display; degraded mode banner shown.
EC-OPS-10: scheduled chaos drill activates during actual election day by mistake; kill switch halts drill within 10 s; all simulated faults reverted; audit trail records drill-abort event.
EC-SCL-01: election-day burst of 10k concurrent API requests; horizontal pod autoscaler adds replicas within 60 s; p95 maintained; no 503 returned after scale-up.
EC-SCL-02: alert backlog of 50k unprocessed events after feed reconnect; consumer processes backlog without dropping events; lag metric decreases monotonically.
EC-SCL-03: alert hot partition caused by single high-volume station; partition key rebalanced to station_code hash; no single consumer overloaded.
EC-SCL-05: database connection pool exhausted under peak load; new connections queued with 2 s timeout; 503 returned with Retry-After if queue full; pool size auto-adjusts within limits.
EC-SCL-06: large media briefing PDF export (> 50 MB) triggered by media desk; async job queued; officer notified via WebSocket on completion; no request timeout.
EC-SCL-07: 250k dispatch records created on election day; p95 write latency <= 2 s; no lock contention on audit log table.
EC-SCL-08: WebSocket fan-out to 500 concurrent ECC sessions on single alert event; message delivered to all sessions within 2 s; no dropped connections.
EC-SEC-01: HSM returns signature mismatch on s.44A commit attempt; operation blocked; incident auto-created; HSM error logged with correlation ID.
EC-SEC-02: brute-force attempt on officer login detected (10 failures in 60 s); account locked; SOC alerted; IP blocked at WAF for 1 h.
EC-SEC-03: HSM becomes unavailable during election day; all s.44A operations and report signing blocked; alert sent to SOC; degraded-mode banner shown on ECC.
EC-SEC-04: API request arrives with forged JWT (invalid signature); rejected with 401; SOC event emitted with actor IP.
EC-SEC-06: SQL injection attempt in alert filter parameter; parameterised query prevents execution; 400 returned; WAF blocks source IP.
EC-SEC-07: canary token accessed in restricted alert detail; SOC alert fires within 30 s with accessor identity.
EC-SEC-08: officer downloads full tally export without export permission in SpiceDB; request denied with 403; event logged.
EC-SEC-09: mTLS certificate on M15 feed revoked; ECC rejects subsequent feed messages; feed-lost alert raised; SOC notified.
EC-SEC-10: session fixation attempt: attacker pre-sets session cookie before officer logs in; server rotates session ID on auth; fixed cookie rejected.
EC-SEC-11: CSRF token missing on state-mutating API call from web UI; request rejected with 403; no action taken.
EC-SEC-12: Semgrep detects hardcoded credential in ECC service code during CI; pipeline fails; commit blocked; secret invalidated.
EC-SEC-13: secret rotation event received from OpenBao; ECC reloads credentials without restart; zero downtime; old credential rejected after 60 s grace.
EC-SEC-14: rate limit exceeded on alert-creation API by external integration; 429 returned with Retry-After; excess requests dropped; integration notified.
EC-SEC-15: replay attack: attacker resends signed s.44A request with valid signature; nonce already consumed; operation rejected with 409.
EC-SEC-17: officer role changed (demoted) while active session open; next API call returns 403; session invalidated within 10 s of role change propagation from SpiceDB.
EC-HUM-01: officer navigates entire ECC dashboard using keyboard only; all actions reachable; focus order logical; no keyboard trap.
EC-HUM-02: screen reader user receives live alert card updates via ARIA live region; each new alert announced within 2 s of arrival.
EC-HUM-03: officer under high stress misclicks "close incident" on wrong incident; confirmation dialog requires typing incident ID; accidental close prevented.
EC-HUM-04: mobile companion app used by field supervisor with visual impairment; large-text mode; high-contrast theme; TalkBack/VoiceOver compatible.
EC-HUM-05: broadcast message composed in Kiswahili with right-to-left numeral mixing; layout correct; no text truncation.
EC-HUM-06: officer working 16 h shift; session idle for 5 min; soft timeout warning shown; 1 min to interact before auto-lock; no unsaved form data lost.
EC-HUM-07: new RO first login; contextual onboarding tooltips shown; dismissible; do not block alert feed.
EC-HUM-08: alert description arrives in 11 languages; UI displays language tag; officer can switch display language without page reload.
EC-HUM-09: poor network on officer laptop (2G simulation); dashboard degrades gracefully; critical alerts still delivered; non-critical components show skeleton loaders.
EC-HUM-10: officer accidentally triggers degradation-mode activation; second officer sees confirmation request with explicit consequence description before approving.
EC-HUM-11: media desk officer attempts to publish briefing before dual-sign is complete; publish button disabled with tooltip explaining missing signatures.
EC-HUM-12: ECC alert sound notification clashes with screen reader audio; officer can disable sound independently; visual indicator remains.
EC-LEG-02: injunction received from court affecting 3 stations; ECC suppresses tally display for those stations immediately; affected stations shown with injunction badge; legal officer notified.
EC-LEG-03: candidate petition filed within 7 days of result publication; petition tracker records filing; tally locked for amendment pending outcome; audit trail immutable.
EC-ML-07: anomaly-detection model (M26) flags a tally as suspicious with high confidence score; ECC surfaces flag with evidence; human officer must acknowledge before tally included in national aggregate.
EC-MOB-01: mobile companion app loses connectivity for 30 min during patrol; all received alerts and dispatch actions cached locally; on reconnect, sync completes within 60 s; no duplicates.
EC-MOB-03: mobile app receives simultaneous push notification and in-app WebSocket alert for the same event; deduplication applied; officer sees single notification with combined context.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | alert aggregation, dispatch FSM, s44A dual-officer flow, NIS loop debounce, RBAC policy | nextest + Flutter test | M16-UNIT- |
| property | alert always reaches terminal state; NIS latency <= 30 s under load; distress <= 5 s | proptest | M16-PROP- |
| contract | M06 telemetry (consumer); M17 speech flags (consumer); M26 anomaly (consumer); M33 dispatch (provider) | Pact | M16-PACT- |
| integration | SoftHSM s44A; all source doubles; distress routing | nextest + testcontainers | M16-INT- |
| e2e | distress -> ECC alert -> dispatch -> ack; s44A dual sign | ExUnit + patrol | M16-E2E- |
| fuzz | alert payload; dispatch instruction; s44A reason | cargo-fuzz | M16-FUZZ- |
| load | 250k dispatches; alert storm; 8-role RBAC | k6 | M16-LOAD- |
| chaos | source feed kill; HSM kill; SpiceDB kill | Chaos Mesh | M16-CHAOS- |
| security | RBAC bypass; s44A single-officer; ASVS L3; Semgrep | Semgrep | M16-SEC- |

## Runbooks and dashboards

- docs/runbooks/ecc-svc-degradation.md (create): source feed disconnect; HSM fail; SpiceDB outage; alert storm mitigation.
- docs/runbooks/ecc-s44a-procedure.md (create): dual-officer s.44A authorisation steps; remote ceremony fallback.
- platform/observability/ecc-svc/grafana-ecc.json: NIS loop latency, alert throughput, dispatch ack rate, feed lag by source.
- platform/observability/ecc-svc/slo-alerts.yaml: 99.9% burn rate; NIS loop > 30 s; distress > 5 s; alert unresolved > 2 h.

## Open questions

- RBAC role matrix: the 8+ roles (national, regional, county, field, legal, IAB, press, observer) need formal sign-off from the Commission HR and legal before implementation.
- NIS loop aggregation: consolidating 14+ source feeds into a single coherent alert model needs an architecture decision on priority weighting; current design uses static severity weights.
