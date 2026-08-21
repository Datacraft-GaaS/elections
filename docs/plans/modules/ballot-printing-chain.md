# M39 ballot-printing-chain

| field | value |
|---|---|
| M-number | M39 |
| Home service | inventory-svc (ADR-0020) |
| Owning work package | WP-4.3 (docs/plans/05-wave-4-operations.md) |
| Size | L |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M39-ballot-printing-chain.md |

## Purpose

Track every ballot paper from print-run authorisation through serial-range gazettement, RFID chip embedding, printer MES lot-data integration, port clearance, Nairobi central warehouse, constituency distribution, and individual polling-station scan-in and scan-out. Make ballot-chain tampering computationally detectable and immediately auditable. The 2017 Kenya election environment in which chain-of-custody weakness was exploited makes integrity here non-negotiable; this module is an explicit corrective response.

Strategy reference: §XV.4 Element 2 (ballot security), §XVI.3 (chain-of-custody named threat).

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Print Authorisation Console | Phoenix LiveView | services/inventory-svc/ex/ballot_chain_web/ |
| RFID Scanner App (M08 integration) | Flutter | services/inventory-svc/apps/rfid-scanner/ |
| Audit Dashboard | Phoenix LiveView | services/inventory-svc/ex/ballot_chain_web/ |
| API | Rust (axum) | services/inventory-svc/crates/inventory-api/ |

Languages: EN only (internal operational system).

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| PrintOrder | id, election_id, contest_id, constituency_code, quantity_authorised, serial_range_start, serial_range_end, paper_spec_hash, issued_by_officer_id, countersigned_by_commissioner_id, issued_at, merkle_node_id, gazette_notice_id | A | T1-audit |
| LotData | id, print_order_id, printer_facility_id, serial_range_start, serial_range_end, quantity_printed, quantity_rejected, rfid_chip_id_list_hash (SHA-256 of sorted chip ID list; full list stored encrypted at rest), print_date, received_at, variance_pct, variance_alert_sent | A | T1-audit |
| CustodyEvent | id, lot_id, event_type (PORT_CLEARANCE/WAREHOUSE_RECEIPT/CONSTITUENCY_ALLOCATION/STATION_RECEIPT/STATION_CLOSE_RETURN/RFID_SPOT_CHECK), location_code, officer_id, witness_id, quantity_scanned, rfid_match_count, rfid_mismatch_count, seal_number, vehicle_registration, scanned_at, merkle_node_id | A | T1-audit |
| StationReconciliation | id, station_code, election_id, ballots_received, ballots_issued, ballots_unused, ballots_spoilt, discrepancy, discrepancy_flag, reconciled_at, reconciled_by, merkle_node_id | A | T1-audit |

## APIs

OpenAPI file: `packages/openapi-contracts/inventory/ballot-chain.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/ballot-chain/print-orders | 2 | JWT+SpiceDB (IEBC_OFFICER) + HSM dual-sign | yes | issue signed print order; dual sign-off enforced |
| POST /v1/ballot-chain/lot-data | internal gRPC | PRINTER_SYSTEM mTLS | yes | printer MES lot-data submission |
| POST /v1/ballot-chain/events | 2 | JWT+SpiceDB (IEBC_OFFICER) | yes | record custody event (scan-in or scan-out) |
| GET /v1/ballot-chain/events | 2 | JWT+SpiceDB (IEBC_AUDIT_LEAD) | n/a | list custody events; filterable by lot and location |
| GET /v1/ballot-chain/events/{id}/proof | 2 | JWT+SpiceDB (IEBC_AUDIT_LEAD) | n/a | Merkle inclusion proof for event |
| POST /v1/ballot-chain/reconciliation | 2 | JWT+SpiceDB (PRESIDING_OFFICER) | yes | submit station end-of-day reconciliation |
| GET /v1/ballot-chain/reconciliation/{station_code} | 2 | JWT+SpiceDB (IEBC_AUDIT_LEAD) | n/a | get station reconciliation record |
| GET /v1/ballot-chain/public/summary | 1 | public | n/a | aggregate chain-of-custody status; no serial IDs or chip IDs exposed |
| GET /v1/ballot-chain/court-bundle/{constituency_code} | 2 | JWT+SpiceDB (COURT_REGISTRAR) | n/a | signed chain-of-custody bundle for M40; rate limited 5/hr |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.inventory.lot_data_received.v1 | emits | eel.inventory.ballot-chain.v1 | audit | yes | at-least-once idempotent |
| eel.inventory.custody_event_recorded.v1 | emits | eel.inventory.ballot-chain.v1 | audit | yes | at-least-once idempotent |
| eel.inventory.quantity_variance_alert.v1 | emits | eel.inventory.ballot-chain.v1 | audit | yes | exactly-once |
| eel.inventory.rfid_mismatch_alert.v1 | emits | eel.inventory.ballot-chain.v1 | audit | yes | exactly-once |
| eel.inventory.station_reconciliation_discrepancy.v1 | emits | eel.inventory.ballot-chain.v1 | audit | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| Printer MES (Al Ghurair UAE / approved alternative) | external mTLS | integrations/printer-mes/ | yes (stub) | lot-data submission; if unavailable, IEBC officer uploads scanned attestation |
| M08 RTS Device | internal | integrations/rts-device/ | yes | RFID scanner at station; offline scan queue; sync on reconnect |
| M06 BVI Back-End | internal | integrations/bvi/ | yes | BVI turnout figure for station reconciliation |
| M16 Election Command Centre | internal | integrations/ecc/ | yes | discrepancy alert consumer; circuit breaker; alert persists in queue |
| M40 Election Court IT Interface | internal | integrations/court/ | yes | court chain-of-custody bundle delivery |
| M11 Public Portal | internal | integrations/portal/ | yes | public aggregate summary publication |
| M15 IAB Dashboard | internal | integrations/iab/ | yes | audit summary; non-critical |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | all events anchored within 60 s |
| P06 HSM | P06 | integrations/hsm/ | yes (stub) | print order dual signing |

## ML and AI

- None.

## Dependencies

- Modules: M06, M08, M11, M15, M16, M40
- Platform: P05, P06
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs

## Feature checklist

1. FR-M39-001: IEBC Print Authorisation Officer issues a signed digital print order; dual sign-off required (officer + IEBC Commissioner); order anchored in P05 Merkle chain before printer receives it
2. FR-M39-002: authorised serial ranges gazetted as Kenya Gazette supplement (PDF + structured JSON); gazette hash anchored in Merkle chain; publicly verifiable
3. FR-M39-003: printer MES submits lot-data JSON (serial ranges, RFID chip IDs, print date, quantity, reject count) via mTLS; system reconciles against authorised print order; quantity variance > 0.01% triggers immediate IEBC alert
4. FR-M39-004: printer attests RFID embedding per ballot in declared range; IEBC spot-check result uploaded to custody record
5. FR-M39-005: IEBC Customs Officer records carrier, AWB/BL number, seal numbers, container ID, port of entry, arrival date, C-63 customs form ID on arrival manifest
6. FR-M39-006: port clearance RFID spot-check minimum 200 chips per lot; discrepancy triggers hold and IEBC Commissioner notification
7. FR-M39-007: Warehouse Manager scans all pallet RFID tags on arrival; system confirms serial-range completeness; missing range segment flagged; signed warehouse receipt anchored
8. FR-M39-008: County Logistics Officer creates constituency allocation record; officer and witness scan-out ballot packets; vehicle registration and physical seal number recorded; allocation signed by both
9. FR-M39-009: Presiding Officer scans ballot packet RFID on delivery at station; system confirms serial range matches allocation; signed receipt generated
10. FR-M39-010: at close, Presiding Officer scans unused ballot bundle RFID; system computes issued = received - unused - spoilt; discrepancy vs BVI turnout count flagged immediately to M16
11. FR-M39-011: post-election return; unused ballots scanned in at constituency store; per-station reconciliation report generated (received/issued/unused/spoilt/discrepancy)
12. FR-M39-012: every scan event, allocation, and reconciliation record hashed and anchored in P05 Merkle chain within 60 seconds of creation
13. FR-M39-013: public summary (aggregate lot quantities, gazette reference, warehouse receipt date, constituency allocation status) on M11; no individual RFID chip IDs published
14. Full RFID chip ID list restricted to IEBC_AUDIT_LEAD role; stored encrypted AES-256; printer-MES connection over mTLS + PGP-encrypted payload

## Acceptance criteria

1. AC-M39-01: print order dual-signed, transmitted to printer, Merkle-anchored; UAT with mock printer MES
2. AC-M39-02: lot-data reconciliation completes within 5 minutes of MES submission; quantity variance alert fires within 1 minute; integration test
3. AC-M39-03: port RFID spot-check (200 chips) logged and anchored; mismatch triggers hold; integration test
4. AC-M39-04: station receipt and close-of-poll reconciliation via M08 RFID scanner; discrepancy alert to M16 within 2 minutes; end-to-end test
5. AC-M39-05: court bundle (all custody events + Merkle proofs) generated within 2 hours of M40 request; integration test
6. AC-M39-06: public summary exposes no serial ranges or chip IDs; penetration test

## Applicable edge cases

**connectivity**
- EC-CON-01: M08 RFID scanner offline at station receipt; local event queue; sync on reconnect; no duplicate events

**security**
- EC-SEC-08: RFID chip ID mismatch at port; shipment held; full re-scan required before Commissioner authorises release; automated alert sent
- EC-SEC-01: printer MES submits without valid mTLS cert; request rejected with 401; logged

**operations**
- EC-OPS-05: RFID scanner failure at station on election day; M08 falls back to manual serial-range record; post-election reconciliation possible via manual tally; discrepancy flag raised for audit review

**scale**
- EC-SCL-01: 290+ constituency allocations processed simultaneously during distribution week; inventory-svc horizontally scaled

**legal**
- EC-LEG-04: printer substitution without IEBC approval; print order specifies approved facility ID; lot-data from unapproved mTLS cert triggers immediate Commissioner notification

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | serial-range completeness checker, RFID chip ID list hash verifier, discrepancy calculator, Merkle anchor trigger, variance alert threshold | nextest | M39-UNIT- |
| property | any committed scan record modification detectable via Merkle proof | proptest | M39-PROP- |
| contract | printer MES mTLS, M08 RFID scan event, M06 BVI turnout, M16 alert, M40 court bundle | Pact | M39-PACT- |
| integration | print order to lot-data reconciliation; station receipt to close-of-poll reconciliation; discrepancy alert; court bundle assembly | integration_test | M39-INT- |
| e2e | full lifecycle: print order to station reconciliation; Merkle proof for every event; court bundle verification | nextest e2e | M39-E2E- |
| security | MES without mTLS cert (reject); officer modifies committed event (Merkle detects); non-audit-lead accesses full chip ID list (deny) | ZAP + red team | M39-SEC- |
| load | 290 constituency allocations concurrently during distribution week | k6 | M39-LOAD- |
| chaos | M08 offline at station receipt; local queue; reconnect; no duplicates | netem | M39-CHAOS- |

Tags: `#[trace("FR-M39-012","AC-M39-04","EC-CON-01")]`

## Runbooks and dashboards

- `docs/runbooks/inventory-svc-degradation.md` - extend with ballot-chain offline queue paths and printer MES unavailability
- `docs/runbooks/ballot-discrepancy-response.md` - new runbook; discrepancy alert escalation; IEBC Commissioner notification; station hold procedure
- Grafana dashboard: `eel-iems-39` (alert ruleset: `iems-39-alerts`)
- SLO alerts: Merkle anchor latency <= 60 s; lot reconciliation <= 5 min; discrepancy alert <= 2 min to M16

## Open questions

- Printer MES API contract: Al Ghurair (UAE) or approved Kenyan alternative; API spec must be co-designed at contract signature; fallback to manual scanned attestation if MES API not delivered by T-270.
- Gazettement format: Attorney General sign-off on serial-range gazette format required by T-180; legal confirmation needed before FR-M39-002 is implemented.
- RFID firmware on M08 kit: M08 hardware must support RFID scanner; confirm scanner firmware update timeline (target T-90) with M08 team.
