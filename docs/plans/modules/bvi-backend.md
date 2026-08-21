# M06 bvi-backend

| field | value |
|---|---|
| M-number | M06 |
| Home service | bvi-svc (ADR-0020) |
| Owning work package | WP-3.7 (docs/plans/04-wave-3-critical-path.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M06-bvi-backend.md |

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

1. FR-M06-001: generate per-station register-extract config bundle at kit-prep time; refresh on demand.
2. FR-M06-002: distribute config bundles to kits via mTLS from Kit Gateway.
3. FR-M06-003: kit whitelist/blacklist management; remote wipe of rogue kit; <= 60 s propagation.
4. FR-M06-004: ingest identification events; verify per-event signature + kit certificate.
5. FR-M06-005: live turnout dashboard: station/ward/constituency/county/national rollup.
6. FR-M06-006: live kit-health dashboard: battery, connectivity, GPS fix, temperature, error counts.
7. FR-M06-007: session lifecycle: PO opens -> window -> PO closes; all transitions audited.
8. FR-M06-008: emergency-close records justification + Elections Act s.55 reference.
9. FR-M06-009: distress-signal handler routes to war-room silently within 5 s.
10. FR-M06-010: per-election configuration: opening/closing time, quorum rules, geo-fence polygon updates.
11. FR-M06-011: kit lost/stolen: revoke certificate; propagate <= 60 s; alert.
12. FR-M06-012: voter-not-found: auto-search central register; return redirect info to kit.
13. FR-M06-013: party-agent aggregate turnout view; no PII; no biometric; rate-limited.
14. FR-M06-014: every state change signed + Merkle-logged + escrow-mirrored.
15. FR-M06-015: publish live turnout to Public Portal (M11) with delay/aggregation policy.
16. Ingest 60 000 kits x 700 events/day = 42M events/day; peak 5 000 events/sec.
17. Turnout dashboard latency <= 30 s from event to dashboard.
18. RTO <= 30 min; RPO <= 5 min.
19. 99.99% availability during polling window.
20. Register-extract refresh triggered within 60 s of BVR register change.
21. Kit telemetry last-seen alert fires if kit silent for > 30 min during polling window.
22. Dual-control required for bulk kit wipe (>1 kit).
23. Turnout feed delay/aggregation policy configurable; default 60 s delay.
24. Party-agent view capped at constituency scope; no cross-constituency access.

## Acceptance criteria

AC-M06-01: 5 000 events/sec sustained without loss (load test).
AC-M06-02: turnout dashboard lag <= 30 s from event to display (field trial).
AC-M06-03: kit revocation propagates in <= 60 s (chaos exercise).
AC-M06-04: emergency-close logged with s.55 justification (UAT).
AC-M06-05: voter-not-found returns correct redirect station (UAT).
AC-M06-06: distress signal reaches war-room silently within 5 s (red-team).
AC-M06-07: 99.99% availability over polling window (SLO measurement).
AC-M06-08: register-extract bundle verified against BVR hash at kit boot (integration test).

## Applicable edge cases

**connectivity**
EC-CON-04: primary/mirror partition; no split-brain; RPO <= 5 min.
EC-CON-11: Kafka ISR shrink; producer blocks; outbox accumulates.

**time**
EC-TIM-03: backend clock backward; HLC never regresses.
EC-TIM-07: s.14A close time moves; HSM-signed extension reopens fence.

**identity**
EC-IDN-01: kit cert revoked mid-session; refuse within 60 s.
EC-IDN-03: SVID rotates during batch; old SVID valid to expiry.
EC-IDN-09: dual-control for bulk wipe; re-validate both at commit.
EC-IDN-10: SpiceDB down; fail closed writes; class-C reads only.

**data**
EC-DAT-01: duplicate identification event same key; cached.
EC-DAT-04: replayed signed event; reject on counter.
EC-DAT-05: votes > registered voters; DLQ with reason.
EC-DAT-14: oversized event; outbox rejects >1 MB.

**security**
EC-SEC-01: signature mismatch; never applied; tamper topic.
EC-SEC-03: HSM unavailable; STH pauses; ingest continues.
EC-SEC-10: voter enumeration; constant-time; caps.
EC-SEC-12: cache poisoning of turnout snapshots.
EC-SEC-13: split-view on log; witness divergence.

**scale**
EC-SCL-03: hot partition; stream-id spread; lag alert; HPA.
EC-SCL-04: consumer lag >60 s; backpressure; no OOM.
EC-SCL-07: 18 000 req/s BVI SLA; p95 800 ms; p99 1.5 s.
EC-SCL-08: Valkey eviction; PG-durable store.

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
- Turnout feed delay policy: IEBC policy on delay/aggregation for the public portal needs formal sign-off (anti-gaming vs transparency).
