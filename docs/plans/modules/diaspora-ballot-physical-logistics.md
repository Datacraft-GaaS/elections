# M65 diaspora-ballot-physical-logistics

| field | value |
|---|---|
| M-number | M65 |
| Home service | inventory-svc (ADR-0020) |
| Owning work package | WP-4.3 (docs/plans/05-wave-4-operations.md) |
| Size | L |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M65-diaspora-ballot-physical-logistics.md |

## Purpose

Manages the physical custody chain for diaspora ballot materials from IEBC headquarters to embassy stations and back, including pre-clearance manifests, offline packing app, customs and airline cargo integration, and reconciliation with the M24 diaspora voting module.

## Surfaces and languages

| surface | technology | path under services/inventory-svc/ |
|---|---|---|
| Admin logistics console | Django | django/inventory-admin/ |
| Offline packing mobile app | Flutter | apps/custody-mobile/ |
| Internal API | Rust (axum) | crates/diaspora-logistics/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| BallotContainer | id, container_code, station_id (embassy), contents_manifest_url, seal_id, status (packed/cleared/in-transit/received/reconciled), packed_at | C | T1-consensus |
| CustodyEvent | id, container_id, event_type (packed/customs-cleared/dispatched/received/seal-broken), actor_id, location, timestamp | C | T1-consensus |
| PreClearanceManifest | id, container_id, mfa_ref, kra_ref, airline_awb, status (submitted/approved/rejected), submitted_at | C | T1-audit |
| ReconciliationRecord | id, container_id, expected_count, received_count, discrepancy, resolved_at | C | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/diaspora-logistics/containers | 3 accredited | JWT+SpiceDB | yes | create ballot container |
| PUT /v1/diaspora-logistics/containers/{id}/pack | 3 accredited | JWT+SpiceDB | yes | record packing with manifest |
| POST /v1/diaspora-logistics/containers/{id}/preclearance | 3 accredited | JWT+SpiceDB | yes | submit MFA/KRA pre-clearance |
| PUT /v1/diaspora-logistics/containers/{id}/dispatch | 3 accredited | JWT+SpiceDB | yes | record dispatch with AWB |
| PUT /v1/diaspora-logistics/containers/{id}/receive | 3 accredited | JWT+SpiceDB | yes | record embassy receipt |
| POST /v1/diaspora-logistics/containers/{id}/custody-events | 3 accredited | JWT+SpiceDB | yes | record custody event |
| POST /v1/diaspora-logistics/containers/{id}/reconcile | 3 accredited | JWT+SpiceDB | yes | trigger reconciliation |
| GET /v1/diaspora-logistics/containers | 3 accredited | JWT+SpiceDB | n/a | list containers with status |
| GET /v1/diaspora-logistics/containers/{id}/custody-chain | 3 accredited | JWT+SpiceDB | n/a | full custody chain |

OpenAPI: packages/openapi-contracts/inventory/diaspora-ballot-physical-logistics.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.custody.diaspora_container_dispatched.v1 | emits | eel.custody.diaspora.v1 | operational | no | at-least-once idempotent |
| eel.custody.diaspora_container_received.v1 | emits | eel.custody.diaspora.v1 | operational | no | at-least-once idempotent |
| eel.custody.seal_broken.v1 | emits | eel.custody.diaspora.v1 | operational | no | at-least-once idempotent |
| eel.custody.diaspora_reconciliation_complete.v1 | emits | eel.custody.diaspora.v1 | operational | no | at-least-once idempotent |
| eel.diaspora.embassy_station_registered.v1 | consumes | eel.diaspora.station.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| MFA (Ministry of Foreign Affairs) | mfa (tier B double) | integrations/ | yes | circuit breaker; clearance deferred |
| KRA e-Clearance | kra-eclearance | integrations/kra-eclearance/ | yes | circuit breaker; clearance deferred |
| Kenya Airways / IATA cargo | airline-cargo | integrations/airline-cargo/ | yes | circuit breaker; AWB manual entry fallback |
| M24 diaspora-voting | internal | crates/diaspora-logistics/ | yes | advisory; station register handoff |
| M20 inventory-management | internal | crates/diaspora-logistics/ | yes | advisory; seal lifecycle |
| M16 election-command-centre | internal | crates/diaspora-logistics/ | yes | advisory; seal-break alert |

## ML and AI

none

## Dependencies

- Modules: M24 (embassy station source), M20 (inventory seal), M16 (ECC alerts)
- Platform: P01 (SpiceDB), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py

## Feature checklist

1. FR-M65-001: Create ballot container with contents manifest per embassy station.
2. FR-M65-002: Record packing with manifest hash and seal ID from M20.
3. FR-M65-003: Submit pre-clearance manifest to MFA and KRA e-Clearance.
4. FR-M65-004: Track pre-clearance approval status.
5. FR-M65-005: Record dispatch with airline AWB number.
6. FR-M65-006: Record embassy receipt with station officer signature.
7. FR-M65-007: Record seal-broken event with alert to ECC via M16 (EC-OPS-03).
8. FR-M65-008: Offline packing app works without internet; syncs on reconnect.
9. FR-M65-009: Offline app queue idempotent; no duplicate custody events on sync.
10. FR-M65-010: Reconcile received ballot materials against dispatch manifest.
11. FR-M65-011: Flag discrepancy in reconciliation; alert operations.
12. FR-M65-012: Full custody chain queryable for any container.
13. FR-M65-013: Admin list of containers with filter by station, status, flight date.
14. FR-M65-014: Alert on container not received within 48 h of scheduled arrival.
15. FR-M65-015: Handle customs embargo or refusal with alternative routing alert.
16. FR-M65-016: Audit trail of all container events.
17. FR-M65-017: Custody chain export as signed PDF for IEBC audit.
18. FR-M65-018: Support return-journey custody chain (post-election ballot return to Nairobi).

## Acceptance criteria

1. AC-M65-01: Seal-broken event pages ECC within 60 s; integration test.
2. AC-M65-02: Offline packing sync is idempotent; double-sync produces no duplicate events; test.
3. AC-M65-03: Reconciliation discrepancy flagged; operations alerted; test.
4. AC-M65-04: Pre-clearance circuit breaker triggers on MFA/KRA unavailability; test.
5. AC-M65-05: Full custody chain retrievable from events; property test with synthetic chain.
6. AC-M65-06: API p99 within routine 99.5% SLA.

## Applicable edge cases

**Operations:**
- EC-OPS-03: seal broken in transit; custody event raised; ECC alerted; inventory FSM.

**Security:**
- EC-SEC-02: tamper/chassis event on container seal; custody event; quarantine stream.
- EC-SEC-07: path traversal on manifest URL; sanitised.

**Connectivity:**
- EC-CON-09: IPsec tunnel drop mid-batch sync; resume at sequence; idempotent re-send.
- EC-MOB-01: OS kills offline app mid-sync; resumable; no duplicate.
- EC-CON-07: satellite RTT for remote embassy sync; MTU-safe; compression.

**Data:**
- EC-DAT-01: duplicate custody event submission; idempotent.
- EC-DAT-04: replayed signed event; reject on counter.

**Time:**
- EC-TIM-04: DST on diaspora embassy kits; UTC storage; gazetted TZ display.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Container FSM, custody chain reconstruction, reconciliation arithmetic | nextest | M65-UNIT- |
| property | Custody chain idempotency; reconciliation invariant | proptest | M65-PROP- |
| contract | MFA/KRA/airline doubles; M24 station; M16 ECC | Pact | M65-PACT- |
| integration | Full logistics lifecycle; offline sync; reconciliation | nextest + testcontainers | M65-INT- |
| e2e | Admin packing; pre-clearance; receive; reconcile | Playwright | M65-E2E- |
| fuzz | Manifest payload; custody event fields | cargo-fuzz | M65-FUZZ- |
| security | ZAP; seal-break alert; log scanner | ZAP + Semgrep | M65-SEC- |

Tags: `#[trace("FR-M65-007","AC-M65-01","EC-OPS-03")]`

## Runbooks and dashboards

- docs/runbooks/inventory-svc-degradation.md (extend with diaspora-logistics section)
- docs/runbooks/diaspora-container-seal-break.md (new)
- platform/observability/inventory-svc/diaspora-logistics-dashboard.json
- platform/observability/inventory-svc/slo-alerts.yaml

## Open questions

- Customs embargo risk: if a country refuses ballot imports, alternative routing (DHL/courier) requires separate MoU; MFA engagement needed.
- Return-journey chain: whether spoilt and unused ballots must be returned and reconciled or destroyed in-country; legal opinion from IEBC required.
