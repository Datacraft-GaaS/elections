# M15 iab-dashboard

| field | value |
|---|---|
| M-number | M15 |
| Home service | ecc-svc (ADR-0020) |
| Owning work package | WP-4.1 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M15-iab-dashboard.md |

## Purpose

Independent Audit Board dashboard: give IAB members a real-time, hardware-key-authenticated window into election operations data (reproducibility bundles, inventory status, anomaly flags, complaints), support escalation workflows to IEBC leadership, and produce statutory compliance reports.

## Surfaces and languages

| surface | technology | path under services/ecc-svc/ |
|---|---|---|
| IAB dashboard admin | Elixir (Phoenix LiveView) | ex/iab-live/ |
| IAB API | Rust (axum + utoipa) | crates/iab-api/ |
| Report generator | Rust + Typst | crates/iab-reports/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| IABEscalation | escalation_id, raised_by, category, description, evidence_refs, status, resolved_at | C | T2 |
| Complaint | complaint_id, source_module, complaint_text, severity, status, escalation_id | C | T2 |
| Report | report_id, type, period, content_hash, sig, published_at | D | T1-audit |
| AuditEvent | event_id, actor, action, ts_hlc, sig | C | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/iab/dashboard.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /iab/v1/summary | 3 accredited | JWT+SpiceDB+HW-key | n/a | Live election summary for IAB members |
| GET /iab/v1/bundle/{id} | 3 accredited | JWT+SpiceDB+HW-key | n/a | Reproducibility bundle status from M14 |
| GET /iab/v1/inventory | 3 accredited | JWT+SpiceDB+HW-key | n/a | Inventory status from M20 |
| GET /iab/v1/anomalies | 3 accredited | JWT+SpiceDB+HW-key | n/a | Anomaly flags from M26 |
| POST /iab/v1/escalations | 3 accredited | JWT+SpiceDB+HW-key | yes | Raise escalation |
| PATCH /iab/v1/escalations/{id} | 3 accredited | JWT+SpiceDB+HW-key | yes | Update escalation status |
| POST /iab/v1/reports | 3 accredited | JWT+SpiceDB+HW-key | yes | Publish signed compliance report |
| GET /iab/v1/reports | 3 accredited | JWT+SpiceDB+HW-key | n/a | List published reports |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.iab.escalation_raised.v1 | emits | eel.iab.escalation.v1 | operational | yes | at-least-once idempotent |
| eel.iab.escalation_resolved.v1 | emits | eel.iab.escalation.v1 | operational | yes | at-least-once idempotent |
| eel.iab.report_published.v1 | emits | eel.iab.report.v1 | operational | yes (HSM) | exactly-once |
| eel.transparency.bundle_published.v1 | consumes | eel.transparency.bundle.v1 | operational | yes | exactly-once |
| eel.inventory.asset_status_changed.v1 | consumes | eel.inventory.asset.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| Transparency bundle M14 | internal | integrations/reproducibility/ | yes | cached status; stale labelled |
| Inventory M20 | internal | integrations/inventory/ | yes | cached; stale labelled |
| Anomaly/fraud M26 | internal | integrations/anomaly/ | yes | feed polled; circuit breaker |
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed on report publish |
| Transparency log (P05) | internal | integrations/tlog/ | yes | report anchoring queued |
| Hardware key (P06/FIDO2) | I07 | integrations/hw-key/ | yes (soft FIDO2) | fail closed; no IAB action without HW-key |

## ML and AI

none

## Dependencies

- Modules: M14 (bundle status), M20 (inventory), M26 (anomaly flags)
- Platform: P05 (audit), P06 (HSM + FIDO2 hardware key)
- Packages: eel-crypto-rs, eel-outbox-rs

## Feature checklist

1. FR-M15-001: FIDO2 hardware-key second factor enforced on all IAB dashboard and API endpoints; no session established without successful assertion.
2. FR-M15-002: SpiceDB permission check on every request, binding IAB-officer and Commission-legal roles; unauthorised subjects receive 403.
3. FR-M15-003: real-time election summary panel showing forms received, tokens issued, anomaly count, and inventory status, refreshed every 10 s via LiveView.
4. FR-M15-004: per-module SLO burn-rate tiles with colour-coded thresholds (green/amber/red) derived from eel.ops.slo.v1 events.
5. FR-M15-005: alert count panel grouped by severity (P1/P2/P3) with one-click drill-down to alert detail.
6. FR-M15-006: audit-log integrity status widget: SHA-256 chain verification result and last-verified timestamp per module.
7. FR-M15-007: per-module deep-dive view: paginated event-stream replay with cursor-based navigation and server-side filtering by event type and time range.
8. FR-M15-008: audit-log chain verification on demand: IAB officer triggers re-verification; result anchored to transparency log.
9. FR-M15-009: anomaly flagging: IAB officer marks an event as anomalous; flag recorded in immutable audit log and surfaced on summary panel.
10. FR-M15-010: compliance reporting: ASVS L3 status, ZAP scan results, and Semgrep clean status presented on a dedicated compliance tab.
11. FR-M15-011: export signed audit-log excerpts as PDF and JSON; HSM signature included; export event anchored to transparency log.
12. FR-M15-012: dual-officer sign-off workflow for clearing a high-severity (P1) alert; both officers must authenticate with hardware key before resolution is committed.
13. FR-M15-013: escalation workflow: IAB officer raises an escalation with category, description, and evidence attachments; escalation routed to IEBC leadership via eel.iab.escalation_raised.v1 event.
14. FR-M15-014: escalation status transitions (open, acknowledged, resolved, closed) with mandatory comment at each transition.
15. FR-M15-015: escalation resolution requires dual-officer sign-off when severity is high or critical.
16. FR-M15-016: alert escalation bridge: PagerDuty and OpsGenie integration for P1 alerts; delivery receipt stored in audit log.
17. FR-M15-017: multi-tenant module drill-down: IAB officer can switch between any module's audit stream without re-authenticating.
18. FR-M15-018: immutable IAB action log: every dashboard action (view, export, flag, resolve, escalate) appended to the audit log with HLC timestamp and actor identity.
19. FR-M15-019: every IAB action log entry signed with actor's key and anchored to transparency log within 60 s.
20. FR-M15-020: RBAC self-service blocked; member add/remove requires dual-control: two existing IAB officers must authenticate and approve the change.
21. FR-M15-021: hardware-key provisioning tracking: record key serial, assigned officer, provision date, and revocation date; show logistics status in admin panel.
22. FR-M15-022: hardware-key revocation: single-officer initiates, second officer confirms; key invalidated in FIDO2 registry; audit entry created.
23. FR-M15-023: dashboard load p95 latency target <= 500 ms measured end-to-end at browser; Grafana SLO alert fires at 1 s.
24. FR-M15-024: compliance report generation completes within 60 s for standard report types; timeout triggers error alert and partial result discard.
25. FR-M15-025: 99.9% uptime SLO during election cycle enforced; error-budget burn-rate alert at 2x and 5x thresholds.
26. FR-M15-026: WCAG 2.2 AA compliance on all dashboard surfaces; automated axe-core scan gates CI.
27. FR-M15-027: multilingual support: English and Kiswahili selectable per session; all UI strings externalised to i18n resource files.
28. FR-M15-028: reproducibility-bundle status view: assembly progress, mirror replication count, download count, and integrity check result from M14.
29. FR-M15-029: inventory dashboard: kit count by status (deployed, returned, faulty, lost) sourced from M20; stale data labelled when feed lag exceeds 30 s.
30. FR-M15-030: anomaly and complaint feed from M26: severity-ranked list, filterable by module and category, updated within 30 s of eel.anomaly.flag.v1 event.
31. FR-M15-031: circuit breaker on M26 anomaly feed; on open, feed shows cached data with staleness label and fires an ops alert.
32. FR-M15-032: session timeout after 15 min of inactivity; re-authentication requires hardware key reassertion.
33. FR-M15-033: all API responses include a signed response envelope (HMAC-SHA-256) verifiable by the client.
34. FR-M15-034: rate limiting on export endpoints (max 10 exports per officer per hour) to prevent bulk exfiltration; excess returns 429.

## Acceptance criteria

AC-M15-01: hardware key required; all accredited endpoints return 401 when FIDO2 assertion is absent or fails; security test verifies bypass attempts rejected.
AC-M15-02: escalation event eel.iab.escalation_raised.v1 reaches Kafka topic within 5 s of submission; integration test asserts consumer receives it within 60 s.
AC-M15-03: compliance report PDF carries valid HSM signature verifiable offline; anchor receipt from transparency log attached; integration test checks both.
AC-M15-04: anomaly feed updates within 30 s of a synthetic eel.anomaly.flag.v1 event published in test; integration test measures latency.
AC-M15-05: dual-control sign-off for P1 alert clearance: attempt by single officer returns 403; only after second officer confirms does state transition to resolved; UAT scenario.
AC-M15-06: 99.9% availability SLO measured over 30-day election-cycle window; error-budget burn-rate Grafana alert fires at 2x threshold within 5 min of synthetic downtime.
AC-M15-07: dashboard load p95 latency is under 500 ms under 50 concurrent simulated IAB sessions (k6 load test); p99 stays under 1 s.
AC-M15-08: WCAG 2.2 AA: axe-core CI scan reports zero violations on all dashboard routes; manual screen-reader test passes for escalation workflow.
AC-M15-09: audit-log entry created for every IAB action; entry is signed and anchored to transparency log within 60 s; e2e test verifies chain.
AC-M15-10: hardware-key revocation removes officer session within one request cycle; subsequent request with revoked key returns 401; unit + integration tests.
AC-M15-11: member add/remove rejected when only one officer approves; succeeds after two officers with hardware keys approve; UAT test.
AC-M15-12: English and Kiswahili locale switching persists across page navigation; all visible strings rendered in selected language; e2e test covers escalation workflow in both locales.
AC-M15-13: export rate limiter returns 429 after the 11th export request within one hour for the same officer; integration test.
AC-M15-14: SpiceDB unavailability causes all IAB write endpoints to return 503 with retry-after header; read endpoints serve cached data with staleness header; chaos test.

## Applicable edge cases

EC-DAT-01: duplicate escalation submitted with same idempotency key within 60 s; second request returns 200 with the existing escalation ID, no duplicate record created.
EC-DAT-02: audit-log export request arrives for a time range that spans a log-rotation boundary; exporter stitches segments correctly and signature covers the full range.
EC-SEC-05: access token replayed after expiry; SpiceDB check rejects; dashboard returns 401; no cached session accepted.
EC-SEC-16: IAB officer inadvertently pastes credentials into an evidence-text field; outbound telemetry pipeline refuses to forward the field; field is redacted in the audit log.
EC-IDN-02: IAB officer's identity provider session expires mid-dashboard session; LiveView connection drops; officer shown re-auth prompt rather than stale data.
EC-IDN-10: SpiceDB pod unavailable during an escalation write; service returns 503; client receives retry-after; no partial write committed; escalation retried on recovery.
EC-SCL-04: Kafka consumer lag on eel.iab.escalation.v1 exceeds 500 messages; Grafana alert fires; dashboard shows feed-delayed banner; SLO burn rate recorded.
EC-IDN-09: one of the two approvers in a dual-control sign-off has their hardware key revoked between initiating and confirming; the pending approval is cancelled; both officers must restart the workflow with valid keys.
EC-OPS-04: degradation-mode misconfiguration sets SLO threshold to 0%, causing all modules to show red burn-rate tiles incorrectly; config validation on startup rejects the value and falls back to a safe default.
EC-SEC-01: FIDO2 assertion signature does not match the registered public key; authentication rejected; failed attempt logged; account locked after 5 consecutive failures.
EC-SEC-06: a forged JWT with a modified role claim reaches the API; SpiceDB check uses the token subject, not the role claim; request denied; security event logged.
EC-SEC-08: SQL injection attempt in anomaly filter query parameter; parameterised query prevents execution; 400 returned; security event logged with sanitised input.
EC-SEC-09: SSRF attempt via evidence URL in escalation attachment; URL allowlist checked before fetch; blocked URL returns 422; security event logged.
EC-SEC-17: HSM returns an unexpected error during report signing; sign operation aborted; partial report discarded; alert fires; no unsigned report reaches storage.
EC-HUM-01: IAB officer submits an escalation with a 10 MB evidence attachment that exceeds the 5 MB limit; form validation rejects before upload with a clear Kiswahili and English error message.
EC-HUM-06: officer clicks "Resolve" on a P1 alert before the second officer has authenticated; UI shows a pending-approval state rather than an error page; second-officer prompt is sent via PagerDuty.
EC-HUM-08: dashboard rendered in Kiswahili locale where a translation key is missing; fallback to English string used; missing-key metric incremented; no visible blank label.
EC-OPS-01: the primary Kafka broker restarts during an escalation event publish; outbox retries with exponential back-off; event eventually delivered; no duplicate after idempotent consumer dedup.
EC-OPS-04: ops misconfiguration sets alert-escalation webhook URL to localhost; webhook delivery fails; circuit breaker opens after 3 failures; alert stays in the queue and fires a secondary Grafana alert.
EC-OPS-09: Grafana data source for SLO burn rates is unavailable; dashboard panels show "no data" with a staleness timestamp rather than zero or green; IAB officer sees the degraded-data warning.
EC-SCL-04: during peak-night result ingestion, anomaly-feed consumer lag reaches 2000 messages; dashboard shows feed-delayed banner; SLO burn-rate panel reflects the lag; auto-scale triggered on consumer group.
EC-DAT-01: idempotent report publish: same report_id submitted twice within the idempotency window; second request returns 200 with existing report; no duplicate anchor created.
EC-SEC-05: rate-limit bypass attempt by rotating officer accounts; per-IP rate limiter kicks in at 20 requests per minute; excess returns 429; security event logged.
EC-IDN-02: hardware key firmware update invalidates the FIDO2 credential; officer registration flow triggered; old credential revoked; new credential anchored; audit log records key-rotation event.
EC-NET-01: network partition isolates the IAB dashboard from SpiceDB; all writes fail closed with 503; reads serve cached permissions with a max staleness of 30 s before they also fail closed.
- EC-DAT-03: out-of-order events on this module's consumed topics are buffered until the gap fills or routed to DLQ
- EC-DAT-05: a schema-valid but semantically impossible payload is routed to DLQ with reason, partition continues
- EC-DAT-06: v1 and v2 event schemas coexist during rollout; both accepted
- EC-DAT-09: Unicode names in every text field are NFC-normalised and never truncated mid-grapheme
- EC-DAT-13: soft-deleted rows never resurface in search or exports

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | FIDO2 auth flow, escalation FSM, report generator, anomaly feed sorter | nextest | M15-UNIT- |
| property | escalation always reaches terminal state; report hash stable | proptest | M15-PROP- |
| contract | M14 bundle status (consumer); M20 inventory (consumer); M26 anomaly (consumer) | Pact | M15-PACT- |
| integration | SoftHSM report sign; soft FIDO2; tlog anchor | nextest + testcontainers | M15-INT- |
| e2e | IAB login -> anomaly review -> escalation -> report publish | ExUnit + Playwright | M15-E2E- |
| security | FIDO2 bypass; ASVS L3; Semgrep; dual-control bypass | Semgrep | M15-SEC- |
| load | concurrent IAB members (50); report generation; feed polling | k6 | M15-LOAD- |

## Runbooks and dashboards

- docs/runbooks/iab-dashboard-degradation.md (create): hardware key logistics; HSM fail; report publish failure; emergency escalation.
- docs/runbooks/iab-key-ceremony.md (create): key provisioning, distribution, and revocation ceremony steps.
- platform/observability/ecc-svc/grafana-iab.json: dashboard latency, escalation queue depth, report generate time, anomaly feed lag.
- platform/observability/ecc-svc/slo-alerts.yaml: 99.9% burn rate; escalation unresolved > 4 h; feed lag > 30 s.

## Open questions

- Hardware key logistics: FIDO2 key provisioning for remote IAB members needs a field distribution plan; current lead-time assumption is 8 weeks before election day.
- Report format: the statutory compliance report format is not yet specified by the Elections Act regulations; IEBC legal team needs to confirm structure before M is implemented.
