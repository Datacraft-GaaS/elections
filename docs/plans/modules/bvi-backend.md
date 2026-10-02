# M06 bvi-backend

| field | value |
|---|---|
| M-number | M06 |
| Home service | bvi-svc (ADR-0020) |
| Owning work package | WP-3.7 (docs/plans/04-wave-3-critical-path.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M06-bvi-backend.md |

## Purpose

Central authority for polling-day operations: distribute per-station register-extract bundles, provision kit configurations, ingest identification events, serve live turnout dashboards, and orchestrate remote lock/wipe of compromised kits within 60 seconds of revocation.

## Surfaces and languages

| surface | technology | path under services/bvi-svc/ |
|---|---|---|
| Ingest API + gRPC | Rust (axum + tonic + utoipa) | crates/bvi-api/ |
| Domain logic | Rust | crates/bvi-domain/ |
| Projections / turnout rollups | Rust | crates/bvi-projections/ |
| Background workers (kit whitelist, telemetry) | Rust | crates/bvi-worker/ |
| Live turnout dashboard + admin | Elixir (Phoenix LiveView) | ex/bvi-live/ |
| Public feed (party agent / RO read) | Elixir | ex/bvi-live/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| KitConfigBundle | bundle_id, station_code, voter_list_hash, polygon_hash, config_version, signed_at | C | T2 |
| IdentificationEvent | event_id, voter_id, modality, outcome, ts_hlc, gps_fix, kit_serial, kit_sig | C | T1-audit |
| TurnoutRollup | rollup_id, scope, scope_code, authenticated_count, updated_at | D | T2 |
| KitCert | kit_serial, cert_pem, revoked, revoked_at, revoked_reason | C | T1-audit |
| SessionLifecycle | session_id, kit_serial, station_code, po_id, opened_at, closed_at, close_reason | C | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/bvi/backend.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /bvi/v1/sessions | internal gRPC | SVID mTLS | yes | Kit opens polling session |
| PATCH /bvi/v1/sessions/{id}/close | internal gRPC | SVID mTLS | yes | Kit closes session |
| POST /bvi/v1/identifications | internal gRPC | SVID mTLS | yes | Identification event batch ingest |
| POST /bvi/v1/telemetry | internal gRPC | SVID mTLS | yes | Kit health telemetry |
| POST /bvi/v1/distress | internal gRPC | SVID mTLS | yes | Covert distress signal |
| GET /bvi/v1/config/{station_code} | internal gRPC | SVID mTLS | n/a | Config bundle for kit |
| POST /bvi/v1/kits/{serial}/revoke | 3 accredited | JWT+SpiceDB | yes | Revoke kit certificate; <= 60 s propagation |
| GET /bvi/v1/turnout | 2 live-per-user | JWT | n/a | Live turnout by scope |
| GET /bvi/v1/kits/health | 3 accredited | JWT+SpiceDB | n/a | Kit health dashboard |
| GET /bvi/v1/voters/{id}/station | 3 accredited | JWT+SpiceDB | n/a | Wrong-station voter redirect lookup |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.bvi.session_accepted.v1 | emits | eel.bvi.session.v1 | operational | yes | at-least-once idempotent |
| eel.bvi.voter_identified.v1 | emits | eel.bvi.identification.v1 | operational | yes | at-least-once idempotent |
| eel.bvi.turnout_rollup_updated.v1 | emits | eel.bvi.turnout.v1 | analytics | yes | at-least-once idempotent |
| eel.bve.kit_revoked.v1 | emits | eel.bve.kit.v1 | operational | yes | exactly-once |
| eel.bvi.distress_received.v1 | emits | eel.bvi.security.v1 | operational | yes | exactly-once |
| eel.bvi.s44a_recorded.v1 | emits | eel.bvi.session.v1 | operational | yes | exactly-once |
| eel.bve.kit_attested.v1 | consumes | eel.bve.kit.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.voter_status_changed.v1 | consumes | eel.bvr.voter.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| BVI device (M05) | internal | integrations/bvi-kit/ | yes | kit queues locally; sync on reconnect |
| BVR back-end (M02) | internal | integrations/bvr-backend/ | yes | register lookup cached; wrong-station redirect from cache |
| Kit PKI (P04) | I07 | integrations/kit-pki/ | yes (SoftHSM) | revocation queued; retry; alert if > 5 s |
| Transparency log (P05) | internal | integrations/tlog/ | yes | Merkle anchoring queued |
| Public Portal (M11) | internal | integrations/public-portal/ | yes | turnout feed delayed/aggregated; circuit breaker |
| MDM (O01) | internal | integrations/mdm/ | yes | remote wipe queued |

## ML and AI

none

## Dependencies

- Modules: M02 (register extract source), M05 (BVI device client), M11 (public turnout feed)
- Platform: P04 (kit PKI/SVID), P05 (audit/Merkle), P06 (HSM)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-resilience-rs

## Feature checklist

Register extract and bundle distribution
1. FR-M06-001: generate per-station register-extract config bundle at kit-prep time; bundle includes voter list, station polygon, and config version.
2. FR-M06-002: distribute config bundles to kits via mTLS from Kit Gateway; delivery confirmed by kit ack event.
3. FR-M06-003: register-extract refresh: detect eel.bvr.voter_status_changed.v1; regenerate and push extract to kit within 60 s.
4. FR-M06-004: register-extract integrity: kit verifies bundle hash against signed reference before use; mismatch triggers alert.
5. FR-M06-005: config bundle versioning: monotonically increasing version; kit rejects downgrade.

Kit lifecycle and management
6. FR-M06-006: kit whitelist/blacklist: central management; revoke certificate; propagate eel.bve.kit_revoked.v1 within 60 s.
7. FR-M06-007: remote wipe of rogue kit: MDM command queued; delivery confirmed; dual-control required for bulk wipe of more than 1 kit.
8. FR-M06-008: kit lost/stolen: revoke certificate; propagate revocation event; alert SRE and war-room.
9. FR-M06-009: kit whitelist ack tracking: unacknowledged mode-switch listed in ECC dashboard; alert after 5 min.
10. FR-M06-010: kit telemetry ingest: battery_percent, temperature_celsius, signal_rssi, gps_fix_quality events stored; last-seen per kit.
11. FR-M06-011: telemetry alert: war-room notification if kit silent for > 30 min during polling window.
12. FR-M06-012: honey mode coordination: when kit cert revoked, M06 flags kit as revoked; kit transitions to honey mode on next contact.

Identification event ingest
13. FR-M06-013: ingest identification events from kits; verify per-event kit signature + SVID mTLS certificate before persistence.
14. FR-M06-014: identification event idempotency: duplicate event_id returns cached 200; no re-execution; Valkey-backed with PG fallback.
15. FR-M06-015: anti-replay: reject events with repeated sequence_counter per kit_serial per session; emit eel.bvi.tamper.v1.
16. FR-M06-016: out-of-order event handling: buffer until gap filled or route to DLQ on timeout; projection independent of wall order.
17. FR-M06-017: bvi-svc ingest p99 <= 100 ms at 18 000 req/s sustained; peak 5 000 events/sec without loss or OOM.
18. FR-M06-018: ingest 42M events/day (60 000 kits x 700 events) sustained; Kafka partition spread by station_code hash.

Turnout and dashboards
19. FR-M06-019: live turnout dashboard: station, ward, constituency, county, national rollup; cumulative + real-time; Phoenix LiveView.
20. FR-M06-020: event-to-dashboard latency <= 30 s; LiveView pub/sub for real-time projection updates.
21. FR-M06-021: live kit-health dashboard: battery, connectivity, GPS fix, last-seen, temperature, error counts; heatmap per kit.
22. FR-M06-022: turnout feed to M11: delay/aggregation policy configurable; default 60 s delay; anti-gaming; circuit breaker on M11 failure.
23. FR-M06-023: party-agent aggregate turnout view: per-station counter only; no voter-level rows; capped at constituency scope; rate-limited 100 req/min.

Session lifecycle
24. FR-M06-024: session lifecycle: PO opens via eel.bvi.session_opened.v1; configurable window; PO closes; all transitions audited.
25. FR-M06-025: emergency-close event records justification + Elections Act s.55 reference; dual-officer sign-off.
26. FR-M06-026: distress-signal handler: eel.bvi.distress_received.v1 routed to war-room silently within 5 s; no UI feedback on kit.
27. FR-M06-027: voter-not-found: cross-search BVR central register; return redirect station and GPS within 2 s.

Configuration and per-election management
28. FR-M06-028: per-election configuration: opening/closing time, quorum rules, form templates, geo-fence polygon updates via admin API.
29. FR-M06-029: geo-fence polygon update: only via HSM-signed bundle; unsigned polygon rejected with 422; stale alert sent.
30. FR-M06-030: s.14A time extension: HSM-signed calendar amendment reopens time fence; forged extension rejected.

Audit, Merkle log, and escrow
31. FR-M06-031: every state change signed and emitted on eel.bvi.* topics; at-least-once idempotent delivery via outbox.
32. FR-M06-032: Merkle-log anchoring: every session open/close and identification event batch anchored within 5 min via P05.
33. FR-M06-033: escrow mirror: every signed event mirrored to escrow at session commit; RPO <= 5 min.
34. FR-M06-034: HSM countersign: session-close events countersigned via HSM (P06); HSM unavailable defers signing, not ingest.

Access control and identity
35. FR-M06-035: SpiceDB policy: all admin actions on kits (revoke, wipe, config) require SpiceDB permission check; fail closed on SpiceDB unavailability.
36. FR-M06-036: SVID rotation: old SVID valid until expiry during batch; producer_key_id per event; SPIRE 30 s rotation tolerated.
37. FR-M06-037: realm JWKS refresh: refetch on kid miss for LiveView dashboard JWTs; cached with 5-min TTL.
38. FR-M06-038: dual-control: bulk kit wipe requires two distinct the Commission officer approvals verified at commit time.

Kafka and outbox
39. FR-M06-039: transactional outbox pattern for all domain events; at-least-once delivery; Kafka ISR write with acks=all.
40. FR-M06-040: consumer lag alert: SRE paged if Kafka consumer lag exceeds 60 s on any bvi topic partition.
41. FR-M06-041: hot-partition mitigation: stream_id includes station_code hash for even partition spread; HPA on ingest workers.
42. FR-M06-042: Valkey eviction: idempotency store backed by PG; Valkey cache only; eviction does not cause duplicate ingest.

Availability and DR
43. FR-M06-043: availability election-critical 99.99% during polling window; Patroni failover on primary DC loss.
44. FR-M06-044: RTO <= 30 min; RPO <= 5 min; Patroni + MirrorMaker on primary DC fail; PITR drill in runbook.
45. FR-M06-045: disk-full guard: bounded Kafka queues; 507 response on ingest when disk quota exceeded; SRE alert.
46. FR-M06-046: cold replay: tiered storage <= 10 MB/s throttle; snapshot-then-tail pattern for historical replay jobs.
47. FR-M06-047: schema compatibility: CBOR+zstd event batches from kit accepted alongside JSON; Karapace compat check on deploy.

Security and compliance
48. FR-M06-048: ASVS L3 compliance on all bvi-svc components; security scan in CI.
49. FR-M06-049: secrets: no PII, biometric hashes, tokens, or voter IDs in logs or traces; Tier A/B telemetry fields rejected.
50. FR-M06-050: 60-second revocation propagation: eel.bve.kit_revoked.v1 delivered to all connected kits and confirmed within SLA; unconfirmed kits paged.

## Acceptance criteria

AC-M06-01: 5 000 events/sec sustained without loss or duplicate; ingest workers stable for 1 hour (load test, k6 + Gatling).
AC-M06-02: turnout dashboard lag <= 30 s from identification event to LiveView display (field trial).
AC-M06-03: kit revocation propagates in <= 60 s; kit halts BVI on next request (chaos exercise).
AC-M06-04: emergency-close event carries Elections Act s.55 justification; dual-officer sign verified (UAT).
AC-M06-05: voter-not-found returns correct redirect station within 2 s; BVR cross-search verified (UAT).
AC-M06-06: distress signal reaches war-room silently within 5 s; no kit-side UX change (red-team).
AC-M06-07: 99.99% availability over polling window; SLO alert fires on first burn-rate breach (SLO measurement).
AC-M06-08: register-extract bundle hash verified against BVR signed reference at kit boot; mismatch blocks session (integration test).
AC-M06-09: bvi-svc p99 <= 100 ms at 18 000 req/s during identification event storm; no OOM (load test).
AC-M06-10: Kafka consumer lag does not exceed 60 s on any partition during 5 000 events/sec sustained run (load test).
AC-M06-11: ASVS L3 automated scan passes with no critical findings; zero Tier A/B secrets found in logs (security scan).
AC-M06-12: Patroni failover: primary DC killed; service recovers in <= 30 min; RPO <= 5 min verified via PITR drill (DR harness).
AC-M06-13: HSM unavailable: ingest continues in degraded mode; deferred signatures drain within 5 min of HSM recovery (chaos test).
AC-M06-14: realm key rotation: all active LiveView sessions accept new kid within 60 s of rotation (integration test).

## Applicable edge cases

connectivity
EC-CON-04: primary/mirror site partition: Patroni fencing, no split-brain; MirrorMaker lag alert; RPO <= 5 min verified.
EC-CON-05: flapping kit link: backoff+jitter retry; retry budget; server deduplication on event_id; no duplicate identification.
EC-CON-07: satellite 600 ms+ RTT for remote kits: transport-tuned gRPC timeouts; MTU-safe chunk sizes.
EC-CON-08: DNS poisoned/down on backend: DNSSEC enforced; hardcoded fallback Kafka bootstrap; circuit opens.
EC-CON-09: IPsec tunnel drop mid-batch: ingest resumes at sequence gap; no loss; gap audit logged.
EC-CON-11: Kafka ISR shrink: producer blocks with acks=all; outbox accumulates; SRE paged; no loss on ISR recovery.

time
EC-TIM-02: NTP/NTS unreachable: HLC + monotonic used; time_confidence=low propagated in all events; tolerance window.
EC-TIM-03: backend clock jumps backward: HLC never regresses; skew alert fired; kani HLC merge property verified.
EC-TIM-06: event time earlier than causation time: accepted; skew anomaly logged; projection independent of wall order.
EC-TIM-07: s.14A close time moves 17:00 to 19:00: HSM-signed calendar amendment reopens time fence; forged extension rejected.
EC-TIM-09: register freeze race at T-60/T-30: OPA hard-close at transaction time; all-or-nothing; TLA+ model verified.

identity
EC-IDN-02: JWT expires mid-request: in-flight admin request completes; next request returns 401 + silent Keycloak refresh.
EC-IDN-03: SVID rotates during kit batch: old SVID valid to expiry; producer_key_id per event; SPIRE 30 s rotation tolerated.
EC-IDN-09: dual-control for bulk wipe; approver revoked mid-flow: re-validate both officers at commit; partial approval rejected.
EC-IDN-10: SpiceDB down: admin writes fail closed; kit identification reads served from class-C cached policy.
EC-IDN-11: realm key rotated, stale JWKS: bvi-live refetches JWKS on kid miss; cached with 5-min TTL.

data
EC-DAT-01: duplicate identification event same key+body: cached 200; no double-mark; proptest auto-generated.
EC-DAT-02: same idempotency key, different body: 422 idempotency-key-reuse; kit must retransmit original.
EC-DAT-03: out-of-order event in stream: buffer until gap filled; route to DLQ on timeout; projection wall-order independent.
EC-DAT-04: replayed signed event with stale sequence_counter: rejected; eel.bvi.tamper.v1 emitted; counter never re-used.
EC-DAT-05: authenticated count > station registered voters: DLQ with reason; partition continues; lag returns 0.
EC-DAT-06: schema v1/v2 coexist: CBOR and JSON both accepted; dual-emit; Karapace compat check on deploy.
EC-DAT-07: partial kit batch failure: per-event report; valid events committed; failed events DLQ'd with reason.
EC-DAT-08: 42M row event ops: streaming ingest; checkpoint-resumable; bounded memory; kill-resume test passes.
EC-DAT-12: Debezium slot lag / WAL bloat: alert at 60 s lag; outbox retained; disk guard prevents unbounded growth.
EC-DAT-14: oversized identification event (> 1 MB): outbox rejects; blob route to MinIO + hash reference; unit + fuzz.

operations
EC-OPS-04: dual-control officer absent for bulk wipe: time-boxed ECC remote co-sign; logged; no single-officer bypass.
EC-OPS-07: staff no-show / kit reassignment: credential exactly-once transfer; old credential revoked on reassignment.
EC-OPS-08: disk full / bucket quota: bounded Kafka queues; 507 on ingest; SRE alert; no corruption or data loss.
EC-OPS-09: wrong degradation mode activated: dual sign-off required at T-0; reversible; all actions logged.
EC-OPS-10: PITR restore during petition window: drill; RPO/RTO measured; restore event logged to transparency log.

scale
EC-SCL-01: 5M turnout dashboard requests/hr: >= 95% served from CDN edge; origin p99 <= 500 ms; k6 + Gatling 5 regions.
EC-SCL-02: thundering herd on STH: SDK jitter on status poll; ETag 304 for unchanged rollup; rate-limit enforced.
EC-SCL-03: hot partition: stream_id includes station_code hash for even spread; lag alert; HPA on ingest workers.
EC-SCL-04: consumer lag > 60 s: SRE paged; backpressure applied; no OOM on slow consumer.
EC-SCL-05: sequencer stall: ingest workers unaffected; cadence alert; drain rate >= peak intake on recovery.
EC-SCL-06: tiered storage lag / cold replay: <= 10 MB/s throttle; snapshot-then-tail; memory-bounded.
EC-SCL-07: 18 000 req/s BVI SLA: p99 <= 100 ms at 1x; <= 10% p99 regression at 5x load; simulator pre-prod.
EC-SCL-08: Valkey eviction loses idempotency key: PG-durable idempotency store; Valkey cache only; flush mid-test verified.

security
EC-SEC-01: kit event signature mismatch: event never applied; tamper topic entry; SRE page within 30 s.
EC-SEC-02: tamper/chassis breach on kit: kill-on-open propagated to backend; quarantine event; ECC alert.
EC-SEC-03: HSM unavailable: STH pauses; ingest continues; countersigns queued; no gap on HSM recovery.
EC-SEC-04: KEK rotation mid-cycle: online re-wrap; old key readable until rotation complete; audit logged.
EC-SEC-05: injection on admin API and config inputs: parameterised queries only; Semgrep; sqlx compile-time; ZAP; fuzz.
EC-SEC-06: SSRF via config URL fields: domain allowlist; no private ranges; DNS rebinding guard; Nuclei scan.
EC-SEC-07: path traversal on register-extract file paths: sanitised; fuzz extractor; ../ corpus.
EC-SEC-08: XXE / deserialization in bundle or event parser: DTD off; no pickle; strict serde models; fuzz parsers.
EC-SEC-09: rate-limit evasion on admin or party-agent APIs: key by subject + device + /64; priority shed; Nuclei.
EC-SEC-10: voter enumeration via wrong-station lookup or turnout API: constant-time shape; caps; step-up on repeated misses.
EC-SEC-11: timing side-channel on signature verification: constant-time compare; dudect-style harness; Semgrep ban naive ==.
EC-SEC-12: cache poisoning of turnout snapshots: cache key includes signature version; LiveView client verifies before render.
EC-SEC-13: split-view on Merkle log: witness divergence detected; consistency check fails; SRE alert.
EC-SEC-14: stolen kit honey mode: revoked cert in backend flags kit; sandbox served; attacker biometric captured.
EC-SEC-15: LiveView admin session hijack: CSRF + session binding; re-auth on step-up admin actions.
EC-SEC-16: secrets in logs/traces: telemetry refuses Tier A/B fields; log-scanner fixture in CI; zero findings gate.
EC-SEC-17: PQ hybrid TLS negotiation failure: classical TLS 1.3 only on fallback; never TLS 1.2; testssl.sh verified.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | revocation propagation logic, turnout rollup, session FSM, distress routing, rate-limiter | nextest | M06-UNIT- |
| property | identification events never duplicated; HLC monotonic; turnout rollup eventually consistent | proptest | M06-PROP- |
| contract | M05 kit event producer; M11 turnout feed consumer; M02 register lookup | Pact | M06-PACT- |
| integration | kit event ingest; revocation with SoftHSM; register extract from BVR double | nextest + testcontainers | M06-INT- |
| e2e | end-to-end polling session open -> identifications -> close -> turnout visible | ExUnit + Playwright | M06-E2E- |
| fuzz | identification event payload; revocation request; config bundle | cargo-fuzz | M06-FUZZ- |
| load | 5 000 events/sec; 60 000 concurrent kit streams; 18 000 req/s BVI SLA | k6 + Gatling | M06-LOAD- |
| chaos | primary kill; Kafka ISR shrink; HSM kill; BVR disconnect | Chaos Mesh | M06-CHAOS- |
| security | ASVS L3; STRIDE; Semgrep; revocation timing test <= 60 s | Semgrep + OWASP | M06-SEC- |

## Runbooks and dashboards

- docs/runbooks/bvi-svc-degradation.md (create): kit revocation procedure; turnout ingest lag; BVR extract stale; dual-control wipe.
- platform/observability/bvi-svc/grafana-polling-day.json: identification rate, turnout rollup, kit health heatmap, revocation latency.
- platform/observability/bvi-svc/slo-alerts.yaml: 99.99% burn rate; revocation propagation > 60 s; ingest lag > 30 s.

## Open questions

- 42M events/day sustained throughput: Kafka partition count and retention size need capacity planning before G3.
- Turnout feed delay policy: the Commission policy on delay/aggregation for the public portal needs formal sign-off (anti-gaming vs transparency).
