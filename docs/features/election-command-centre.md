# Election command centre (M16)

Module M16 (`election-command-centre`) owned by `ecc-svc`. Real-time operational dashboard for 46,000+ kits: telemetry render < 2 s, alert dedup + rate-limit + severity routing, Elections Act s.44A dual-control state machine, and a field-dispatch loop.

## Contents

1. What M16 does
2. Data plane
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M16 does

M16 is the operational nerve centre for election day. It aggregates telemetry from 46,000+ KIEMS kits, renders a national map view within 2 s, deduplicates and severity-routes alerts, drives the field-dispatch workflow (kit replacement, officer deployment), and owns the dual-control s.44A invocation state machine (extending polling hours requires Commissioner + Deputy Commissioner biometric co-sign).

## 2. Data plane

Crates: `services/ecc-svc/crates/ecc-api/src/` (handlers.rs, models.rs, router.rs, sse.rs). Elixir Phoenix LiveView: `services/ecc-svc/ex/`.

| Aggregate | Purpose | Data class |
|---|---|---|
| **KitTelemetrySnapshot** | kit_serial, battery_pct, connectivity_signal, queue_depth, enrolment_rate / id_rate, last_seen_ts, geo_lat, geo_lon, status (ONLINE/OFFLINE/DEGRADED/QUARANTINED) | C |
| **AlertEvent** | alert_id, kit_serial (or scope), severity (L1/L2/L3/L4), alert_type, deduplicated, suppressed_until, ts | C |
| **DispatchOrder** | order_id, alert_id, assigned_officer_id, kit_serial_target, action_type (REPLACE_KIT/DEPLOY_OFFICER/RESUPPLY), status, ts | B |
| **S44AInvocation** | invocation_id, scope (station/ward/constituency/national), requested_by, commissioner_sig, deputy_sig, effective_at, new_close_time, ts | A |
| **EccAuditEvent** | event_id, actor, action, ts_hlc, scope | C |

## 3. Ingest sources

| Source | Notes |
|---|---|
| biometric-edge-svc telemetry | `eel.bve.kit_health_reported.v1` Kafka topic; battery, connectivity, queue depth, enrolment rate per kit |
| M05/M06 bvi-backend | Turnout rollup per station (`eel.bvi.turnout_updated.v1`) |
| M08/M10 rts-backend | Form 34A transmission status per station (`eel.rts.form34a_transmitted.v1`) |
| M06 kit certificate lifecycle | Revocation and honey-mode events |
| Geo-violation alerts | `eel_geo_violation` events from any kit |
| IAB escalations (M15) | L3/L4 escalations trigger ECC notification |

## 4. Pipeline

Crates: `services/ecc-svc/crates/ecc-api/src/`, `services/ecc-svc/ex/` (Phoenix LiveView).

```
telemetry stream (Kafka eel.bve.kit_health_reported.v1, eel.bvi.turnout_updated.v1, ...)
      -> ecc-api: ingest to KitTelemetrySnapshot table (upsert by kit_serial)
      -> SSE fan-out (ecc-api/src/sse.rs):
             all connected ECC consoles receive update within 2 s
             Phoenix LiveView subscription over WebSocket
      -> national map render: 46k kit markers updated < 2 s SLO
             marker colour: GREEN (online), AMBER (degraded), RED (offline/quarantined)
             hover: battery, connectivity, queue depth, last seen

alert pipeline:
      kit event classified -> AlertEvent created (severity L1-L4)
      -> dedup: same alert_type + kit_serial within 5 min window -> suppressed
      -> rate-limit: same alert_type across > 100 kits within 60 s -> batched summary
      -> severity routing:
             L1 (informational) -> dashboard only
             L2 (warning)       -> dashboard + operator email
             L3 (critical)      -> dashboard + page + IAB notification
             L4 (emergency)     -> dashboard + page + Commissioner phone + L4 dispatch

dispatch loop:
      operator reviews AlertEvent -> POST /ecc/v1/dispatch
      -> DispatchOrder created; assigned to nearest available officer
      -> officer receives dispatch via mobile-bff (M13) SSE push
      -> officer checks in (M13 dispatch-api/src/routes/checkin.rs)
      -> officer checks out on resolution (checkout.rs)
      -> DispatchOrder.status = RESOLVED

s.44A dual-control state machine:
      Commissioner requests polling hours extension
      -> S44AInvocation created (status = PENDING)
      -> Commissioner biometric-auth + Ed25519 sign
      -> Deputy Commissioner biometric-auth + Ed25519 sign
             both must sign within 10-min window
      -> on both sigs: S44AInvocation.status = APPROVED
      -> HSM-signed extension bundle emitted to all kits via MDM (O01)
      -> kits update time-fence FSM on receipt
      -> eel.ecc.s44a_invoked.v1 emitted to transparency log (P05)
```

## 5. API surface

Contract: `packages/openapi-contracts/ecc/command.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /ecc/v1/telemetry` | Commission operator (JWT + SpiceDB) | All-kit telemetry snapshot |
| `GET /ecc/v1/telemetry/{kit_serial}` | Commission operator (JWT + SpiceDB) | Single-kit detail |
| `GET /ecc/v1/alerts` | Commission operator (JWT + SpiceDB) | Active alert queue |
| `POST /ecc/v1/alerts/{id}/suppress` | Supervisor (JWT + SpiceDB) | Suppress alert with reason |
| `POST /ecc/v1/dispatch` | Operator (JWT + SpiceDB) | Create dispatch order |
| `PATCH /ecc/v1/dispatch/{id}` | Officer (JWT + SpiceDB) | Update dispatch status (checkin/checkout) |
| `POST /ecc/v1/s44a` | Commissioner (JWT + SpiceDB + biometric) | Initiate s.44A polling extension |
| `PATCH /ecc/v1/s44a/{id}/countersign` | Deputy Commissioner (JWT + SpiceDB + biometric) | Countersign s.44A invocation |
| `GET /ecc/v1/sse` | Commission operator (JWT) | SSE stream of all real-time events |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| ECC national map console (Phoenix LiveView) | `services/ecc-svc/ex/` | 46k-kit real-time operational view |
| Alert management panel | Same LiveView | Alert queue, dedup status, suppress, dispatch |
| s.44A panel | Same LiveView | Dual-control extension invocation |
| IAB alert feed (M15) | SSE subscription from ECC | L3+ alerts visible to IAB |

Languages: English (primary operational language for national command). WCAG 2.1 AA for operator surfaces. Map: SVG-based constituency/county drill-down; no external mapping dependency.

## 7. Integrity + anti-tamper

- **s.44A dual-control** -- both Commissioner and Deputy must biometric-auth and sign within a 10-minute window; single-party invocation impossible; HSM-signed extension bundle required at kit; forged bundle rejected.
- **Transparency log anchoring** -- every s.44A invocation emits `eel.ecc.s44a_invoked.v1` to P05; independently verifiable.
- **Alert dedup** -- dedup keyed on `alert_type + kit_serial`; suppression window stored server-side; no alert silently lost; suppressed alerts visible in audit trail.
- **Dispatch audit** -- every DispatchOrder creation and status change logged to EccAuditEvent with actor and ts_hlc.
- **Telemetry source verification** -- KitTelemetrySnapshot only accepted from kits with valid SVID certificates; quarantined kits appear in map but cannot update their own telemetry status.
- **SSE fan-out integrity** -- SSE messages carry snapshot hash; client-side ECC console verifies hash on receipt.
- **No writes to election data** -- ECC has read-only access to M02, M06, M10 data stores; all writes are confined to ECC-owned tables (alerts, dispatch, s.44A).

## 8. Kenya statute alignment

- **Elections Act s.44A (extended polling hours)** -- dual-control state machine implements the statutory requirement for Commissioner authorisation of any polling extension.
- **Elections Act s.39 (result management)** -- ECC monitors Form 34A transmission status per station; missing stations trigger L3 alerts.
- **Elections (Technology) Regulations 2017 Reg. 10 (command and control)** -- ECC is the statutory operations centre; s.44A invocation log provides the audit trail.

## 9. Honest caveats + external prerequisites

- **< 2 s render** with 46,000 kits requires server-sent event fan-out infrastructure scaled to >= 500 concurrent ECC operator sessions; load test required.
- **MDM (O01) push latency** for s.44A extension bundles to 65,000 kits is an MDM infrastructure characteristic; end-to-end propagation time (target < 5 min) must be validated in field trial.
- **ECC map data** (constituency polygons) sourced from boundary-svc (`services/boundary-svc/`); boundary data must be current before election day.
- **Alert dedup thresholds** (5-min window, 100-kit rate-limit) are configurable; must be tuned against actual election-day alert volumes in rehearsal.
- **Officer dispatch mobile coverage** -- field officers must have GSM coverage to receive dispatch SSE push; IVR fallback via Africa's Touching is the contingency.
- `services/ecc-svc` has no CAPABILITIES.md on disk; capability detail is in plan file and crate source.

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (alert dedup, severity router, s.44A FSM, dispatch order FSM) | nextest | M16-UNIT- |
| Property (s.44A requires exactly two sigs; alert dedup window monotone) | proptest | M16-PROP- |
| Contract (kit telemetry ingest; M06 turnout consumer; M10 RTS consumer; IAB alert SSE) | Pact | M16-PACT- |
| Integration (SSE fan-out; s.44A MDM push double; tlog double; SpiceDB) | nextest + testcontainers | M16-INT- |
| Load (46k kit telemetry events/min; 500 concurrent ECC sessions; < 2 s render) | k6 | M16-LOAD- |
| Security (s.44A single-sig bypass attempt; alert suppression escalation; SSE injection) | Manual + ZAP | M16-SEC- |

Runbooks (to create): `docs/runbooks/ecc-svc-degradation.md` (telemetry ingestion backlog; s.44A propagation delay; SSE fan-out failure). Related: `docs/runbooks/election-day-mode-toggle.md`.

Full spec: `docs/plans/modules/election-command-centre.md`. Crates: `services/ecc-svc/crates/ecc-api/`. LiveView: `services/ecc-svc/ex/`. Dispatch: `services/field-mobile-svc/crates/dispatch-api/`.
