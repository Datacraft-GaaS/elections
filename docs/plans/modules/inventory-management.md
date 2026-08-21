# M20 inventory-management

| field | value |
|---|---|
| M-number | M20 |
| Home service | inventory-svc (ADR-0020) |
| Owning work package | WP-4.3 (docs/plans/05-wave-4-operations.md) |
| Size | XL |
| Availability class | cycle-critical 99.9% |
| Event-sourced | yes (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M20-inventory-management.md |

## Purpose

End-to-end asset lifecycle management for all election materials (kits, ballot papers, seals, consumables): track custody from warehouse to polling station and back, enforce seal integrity, manage reserve pools, and integrate with OEM warranty and logistics partners to ensure zero untracked asset on election day.

## Surfaces and languages

| surface | technology | path under services/inventory-svc/ |
|---|---|---|
| Admin web interface | Django on packages/eel-web-py | django/inventory-admin/ |
| Mobile field app (MDM) | Flutter on packages/kit-sdk-flutter | apps/inventory-mobile/ |
| Inventory API | Rust (axum + utoipa) | crates/inventory-api/ |
| Domain + event store | Rust | crates/inventory-domain/ |
| Batch reconciliation | Rust | crates/inventory-batch/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| Asset | asset_id, type (kit/ballot/seal/consumable), serial, model, status, station_code, last_seen_at | C | T1-audit |
| CustodyEvent | event_id, asset_id, from_custodian, to_custodian, location, ts_hlc, sig, seq | C | T1-audit |
| Seal | seal_id, asset_id, seal_type, affixed_at, affixed_by, broken_at, broken_reason, sig | C | T1-audit |
| Consumable | item_id, type, quantity, unit, station_code, replenishment_threshold | C | T2 |
| ReservePool | pool_id, scope, asset_type, count, alert_threshold, updated_at | C | T2 |

## APIs

OpenAPI file: packages/openapi-contracts/inventory/assets.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /inventory/v1/assets | 3 accredited | JWT+SpiceDB | yes | Register asset |
| PATCH /inventory/v1/assets/{id}/custody | 3 accredited | JWT+SpiceDB | yes | Transfer custody; HSM-signed |
| POST /inventory/v1/seals | 3 accredited | JWT+SpiceDB | yes | Affix seal; HSM-signed |
| PATCH /inventory/v1/seals/{id}/break | 3 accredited | JWT+SpiceDB | yes | Break seal with reason; dual-officer |
| GET /inventory/v1/assets | 3 accredited | JWT+SpiceDB | n/a | Asset list with filters |
| GET /inventory/v1/assets/{id}/history | 3 accredited | JWT+SpiceDB | n/a | Full custody chain |
| GET /inventory/v1/reserves | 3 accredited | JWT+SpiceDB | n/a | Reserve pool status |
| GET /inventory/v1/summary | 3 accredited | JWT+SpiceDB | n/a | Election-day asset status summary |
| POST /inventory/v1/reconcile | 3 accredited | JWT+SpiceDB | yes | Trigger batch reconciliation |
| GET /inventory/v1/reconcile/{id}/report | 3 accredited | JWT+SpiceDB | n/a | Reconciliation report |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.inventory.asset_registered.v1 | emits | eel.inventory.asset.v1 | event-source | yes (HSM) | exactly-once |
| eel.inventory.custody_transferred.v1 | emits | eel.inventory.asset.v1 | event-source | yes (HSM) | exactly-once |
| eel.inventory.seal_affixed.v1 | emits | eel.inventory.seal.v1 | event-source | yes (HSM) | exactly-once |
| eel.inventory.seal_broken.v1 | emits | eel.inventory.seal.v1 | event-source | yes (HSM dual) | exactly-once |
| eel.inventory.asset_status_changed.v1 | emits | eel.inventory.asset.v1 | event-source | yes | exactly-once |
| eel.inventory.reserve_alert.v1 | emits | eel.inventory.reserve.v1 | operational | yes | at-least-once idempotent |
| eel.bvi.kit_revoked.v1 | consumes | eel.bve.kit.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed on custody + seal events |
| RFID/barcode readers | I10 | integrations/rfid/ | yes (stub) | manual entry fallback |
| OEM warranty / MES | I10 | integrations/oem/ | yes (stub) | async; alert on API fail |
| Boundary svc M18 | internal | integrations/boundary/ | yes | station geofence for location validate |
| ECC M16 | internal | integrations/ecc/ | yes | reserve alert + asset alert push |
| IAB dashboard M15 | internal | integrations/iab/ | yes | inventory status feed |
| Logistics partners | external | integrations/logistics/ | yes (stub) | shipment tracking; async |

## ML and AI

none

## Dependencies

- Modules: M08 (kit asset source events), M15 (IAB status consumer), M16 (ECC alert consumer), M18 (location boundary validation)
- Platform: P05 (audit), P06 (HSM), RFID infra
- Packages: eel-crypto-rs, eel-outbox-rs, eel-web-py, kit-sdk-flutter

## Feature checklist

1. FR-M20-001: register assets with serial number, type, model, and initial custodian.
2. FR-M20-002: custody transfer with HSM-signed event; dual-officer for high-value assets.
3. FR-M20-003: RFID/barcode scan at transfer points; manual fallback if scanner fails.
4. FR-M20-004: seal lifecycle: affix, verify, break (dual-officer) with HSM-signed events.
5. FR-M20-005: reserve pool management: alert ECC when below threshold.
6. FR-M20-006: consumable tracking: ballot paper count, ink, seals, by station.
7. FR-M20-007: mobile field app: offline-capable; queue events locally; drain on reconnect.
8. FR-M20-008: batch reconciliation: compare physical count vs event-sourced ledger; generate discrepancy report.
9. FR-M20-009: OEM warranty integration: asset registration triggers warranty record creation.
10. FR-M20-010: logistics partner tracking: shipment status linked to asset custody events.
11. FR-M20-011: full custody chain query: any asset's complete history from registration to current location.
12. FR-M20-012: election-day summary: count of deployed/returned/unaccounted assets per scope.
13. FR-M20-013: kit revocation event from M06 triggers asset status update to "revoked".
14. FR-M20-014: geofence validation: custody transfer location validated against M18 polygon.
15. Zero untracked assets at election day start (reconciliation target).
16. Custody transfer HSM sign <= 2 s p95.
17. Batch reconciliation for national election <= 4 h.
18. Reserve alert delivered to ECC within 60 s of threshold breach.
19. Offline mobile queue holds up to 7 days; no data loss on reconnect.
20. Dual-officer required for seal break and high-value transfers.

## Acceptance criteria

AC-M20-01: custody chain verifiable end-to-end for a kit from warehouse to station (integration test).
AC-M20-02: seal break requires dual-officer HSM auth; single officer cannot complete (UAT).
AC-M20-03: batch reconciliation generates discrepancy report with zero false-negatives (test with injected missing asset).
AC-M20-04: reserve alert reaches ECC within 60 s (integration test).
AC-M20-05: offline mobile queue drains without data loss after 7-day accumulation (chaos test).
AC-M20-06: OEM warranty record created on asset registration (integration test).
AC-M20-07: 99.9% availability during election cycle (SLO measurement).

## Applicable edge cases

**connectivity**
EC-CON-01: offline mobile; local event queue; drain on reconnect.
EC-CON-09: tunnel drop mid-custody transfer; resume at event sequence.

**time**
EC-TIM-03: HLC on all custody events; no regression.

**identity**
EC-IDN-01: field officer cert revoked; mobile app refuses custody transfer.
EC-IDN-09: dual-officer: re-validate both at commit; no surrogate.
EC-IDN-10: SpiceDB down; fail closed custody + seal writes.

**data**
EC-DAT-01: duplicate custody event same key; cached; no double-transfer.
EC-DAT-04: replayed signed event; reject on seq counter.

**security**
EC-SEC-01: HSM signature mismatch on custody event; reject; tamper alert.
EC-SEC-03: HSM unavailable; custody + seal blocked; alert.
EC-SEC-02: RFID tag cloned; tamper detection via signed event chain.

**operations**
EC-OPS-01: power cut mid-reconciliation; journal-before-act; resume.
EC-OPS-08: RFID reader fails; manual barcode entry logged; alert on high manual rate.

**scale**
EC-SCL-06: 60k kits + ballot paper counts for 40k stations; batch reconciliation parallelism.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | custody FSM, seal lifecycle, reserve alert trigger, reconciliation diff, geofence validate | nextest + Flutter test | M20-UNIT- |
| property | custody chain append-only; seal state machine; no double-transfer | proptest | M20-PROP- |
| contract | M06 kit-revoked (consumer); M16 reserve alert (provider); M15 status feed (provider) | Pact | M20-PACT- |
| integration | SoftHSM custody + seal; RFID stub; logistics stub; offline drain | nextest + testcontainers | M20-INT- |
| e2e | asset register -> custody chain -> seal affix -> break (dual) -> reconcile | patrol + Playwright | M20-E2E- |
| fuzz | custody transfer payload; seal break reason; reconciliation input | cargo-fuzz | M20-FUZZ- |
| formal | custody chain append-only; no untracked asset in steady state | kani | M20-FORMAL- |
| load | 60k kit transfers + 40k station reconciliations; mobile drain | k6 | M20-LOAD- |
| chaos | HSM kill; SpiceDB kill; RFID reader fail; mobile offline 7 days | Chaos Mesh | M20-CHAOS- |
| security | RFID clone; signature replay; ASVS L3; Semgrep | Semgrep | M20-SEC- |

## Runbooks and dashboards

- docs/runbooks/inventory-svc-degradation.md (create): HSM fail; RFID outage; reconciliation failure; reserve alert escalation.
- docs/runbooks/inventory-seal-ceremony.md (create): seal affix, verify, and break dual-officer ceremony steps.
- platform/observability/inventory-svc/grafana-inventory.json: asset status heatmap, custody transfer rate, seal integrity count, reserve pool levels, reconciliation lag.
- platform/observability/inventory-svc/slo-alerts.yaml: 99.9% burn rate; reconciliation > 4 h; reserve below threshold unacknowledged > 30 min.

## Open questions

- OEM/logistics API availability: OEM warranty APIs and logistics partner tracking APIs are unconfirmed; if unavailable before G4, the integration degrades to manual CSV import.
- RFID lead time: RFID reader procurement and field deployment requires >= 12 months lead time; if not confirmed by T-18 months, barcode scanning is the primary scan method.
