# Ballot printing chain (M39)

Module M39 (`ballot-printing-chain`) owned by `inventory-svc`. Tracks ballot paper from print-run authorisation through RFID/barcode embedding, KRA e-clearance, port-of-entry release, national warehouse, constituency distribution, and polling-station scan-in/scan-out. Every custody transfer is Merkle-anchored and HSM-signed; station reconciliation records spoilage and returns.

## Contents

1. What M39 does (vs M20 inventory management)
2. Data plane (schema)
3. Custody chain phases
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M39 does (vs M20 inventory management)

- **M20** = general inventory management for all election materials (ink, seals, forms, ballot boxes).
- **M39** = high-assurance chain-of-custody exclusively for ballot paper: from printer manufacturing execution system (MES) handshake through post-election return. Every ballot lot carries a UUID7 lot ID embedded in its RFID/barcode; every physical move is a signed `CustodyEvent`.

M39 is the integrity backbone for ballot secrecy and anti-stuffing assurance. It feeds M40 (court evidence bundle) and M06 (BVI ballot-image correlation) with chain-of-custody proofs.

## 2. Data plane

Four aggregates (per `services/inventory-svc/crates/printing/src/models.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **PrintOrder** | Root authorisation: `order_id` (UUID7), `election_cycle`, `contest_ids[]`, `lot_count`, `ballots_per_lot`, `printer_svid`, `mes_order_ref`, `kra_clearance_ref`, `authorised_by` (Commissioner + HSM), `status` (`DRAFT / AUTHORISED / IN_PRINT / CLEARED / DISTRIBUTED / CLOSED`). | A |
| **LotData** | One physical lot: `lot_id` (UUID7), `order_id`, `lot_serial` (MES-assigned), `barcode_range_start`, `barcode_range_end`, `rfid_batch_id`, `ward_allocation` (destination ward code), `print_timestamp`, `mes_double_verified` (boolean). | A |
| **CustodyEvent** | Single physical transfer: `event_id` (UUID7), `lot_id`, `from_location`, `to_location`, `scanned_by` (officer SVID or RFID scanner device ID), `scan_timestamp`, `hsm_signature`, `event_type` (`PRINT_COMPLETE / PORT_CLEARED / WAREHOUSE_IN / WAREHOUSE_OUT / CONST_IN / STATION_IN / STATION_OUT / SPOILAGE / RETURN`). | A |
| **StationReconciliation** | Closing reconciliation per station: `recon_id` (UUID7), `lot_id`, `station_code`, `issued`, `used`, `spoiled`, `returned`, `signed_by` (Presiding Officer SVID), `countersigned_by` (ECC officer SVID), `merkle_anchor_id`. | A |

All tables in `inventory_ballots` schema, migrations under `services/inventory-svc/migrations/`.

## 3. Custody chain phases

| Phase | Location | Event type | Verification |
|---|---|---|---|
| Print authorisation | Printer facility | `PRINT_COMPLETE` via MES webhook | MES double-sign + RFID batch hash |
| KRA e-clearance | Customs | `PORT_CLEARED` via KRA customs API | KRA transaction reference stored in `PrintOrder.kra_clearance_ref` |
| National warehouse in | IEBC warehouse | `WAREHOUSE_IN` via RFID scanner | RFID scan vs lot barcode range; mismatch alerts ops |
| Constituency dispatch | Warehouse out | `WAREHOUSE_OUT` | Officer SVID + lot seal scan |
| Constituency store in | Const. store | `CONST_IN` | RO SVID + RFID scan |
| Polling station delivery | Station | `STATION_IN` | Presiding Officer SVID + RFID scan |
| Post-election return | Station to const. store | `STATION_OUT` + `RETURN` | PO + ECC officer countersignature |
| Spoilage | Station | `SPOILAGE` | PO SVID + reason code; counted in `StationReconciliation.spoiled` |

The MES double-verification (`mes_double_verified`) follows the NIST SP 800-88 Rev.1 standard for media accountability: the printer's MES records lot output, and IEBC's adapter independently queries the MES API for the same lot before emitting `PRINT_COMPLETE`.

## 4. Pipeline

```
Commissioner + HSM authorises PrintOrder (status = AUTHORISED)
      -> emit eel.inventory.print_order_authorised.v1
      -> MES webhook received (printer manufactures lots)
            -> LotData created per lot
            -> CustodyEvent(PRINT_COMPLETE) signed by MES SVID
      -> KRA e-clearance API polled; ref stored
            -> CustodyEvent(PORT_CLEARED)
      -> RFID scanners at IEBC warehouse
            -> WAREHOUSE_IN event per lot; mismatch alert if lot count off
      -> constituency allocation computed (ward_allocation per lot)
      -> WAREHOUSE_OUT + CONST_IN events as trucks depart + arrive
            -> M16 ECC notified via eel.inventory.lot_dispatched.v1
      -> STATION_IN on election day (Presiding Officer RFID scan)
            -> M06 BVI correlation enabled for that lot
      -> post-election: PO completes StationReconciliation
            -> issued / used / spoiled / returned tallied
            -> PO SVID + ECC countersignature
            -> Merkle anchor via P05 (one leaf per StationReconciliation)
            -> eel.inventory.station_recon_complete.v1
      -> CustodyEvent(RETURN) as ballots shipped back to secure store
      -> PrintOrder status -> CLOSED when all lots reconciled
```

Circuit breakers via `packages/eel-resilience-rs` guard MES and KRA API calls. Outbox pattern via `packages/eel-outbox-rs` ensures no event lost.

## 5. API surface

Contract: `packages/openapi-contracts/inventory/ballot-chain.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/ballot-chain/orders` | Commissioner + HSM | Create + authorise PrintOrder |
| `GET /v1/ballot-chain/orders/{order_id}` | Commission admin | Order status + lot count |
| `POST /v1/ballot-chain/orders/{order_id}/lots` | MES SVID | Register LotData batch (MES webhook) |
| `POST /v1/ballot-chain/lots/{lot_id}/events` | RFID scanner device (SVID) or officer (JWT) | Record CustodyEvent |
| `GET /v1/ballot-chain/lots/{lot_id}/chain` | Commission admin / court registrar | Full event chain for one lot |
| `POST /v1/ballot-chain/lots/{lot_id}/reconciliation` | Presiding Officer (JWT) | Submit StationReconciliation |
| `GET /v1/ballot-chain/reconciliations` | ECC officer | List reconciliations pending countersignature |
| `PATCH /v1/ballot-chain/reconciliations/{recon_id}/countersign` | ECC officer (JWT) + HSM | Countersign reconciliation |
| `GET /v1/ballot-chain/lots/{lot_id}/proof` | Court registrar | Merkle inclusion proof for lot |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Ballot chain console (LiveView) | `services/inventory-svc/ex/ballot_chain_web` | National ops view; lot tracking; alert triage |
| RFID scanner app (Flutter) | `services/inventory-svc/apps/rfid-scanner` | Warehouse + station scan-in/out; offline queue |
| Commissioner authorisation panel (Django) | `services/inventory-svc/django/ballot_admin` | HSM-sign print orders; close orders |
| Court evidence export | `services/inventory-svc/ex/ballot_chain_web` | Lot chain export for M40 evidence bundle |

Languages: EN + SW on all staff surfaces. RFID scanner app displays lot ID and station code only; language-neutral.

## 7. Integrity + anti-tamper

- **MES double-verification** (NIST SP 800-88 Rev.1) - IEBC adapter independently queries MES for lot count before accepting `PRINT_COMPLETE`. A discrepancy halts the lot and alerts ops.
- **RFID + barcode duality** - every lot has both an RFID tag (for warehouse scanners) and a printed barcode range (for manual verification if RFID fails). Both are stored in `LotData`; either can trigger a `CustodyEvent`.
- **Merkle-anchored reconciliations** - each `StationReconciliation` is a P05 leaf; inclusion proof available for court proceedings. A gap in the Merkle tree (missing reconciliation) is flagged by the nightly audit job.
- **HSM signatures on all critical events** - `PRINT_COMPLETE`, `STATION_IN`, and all reconciliation countersignatures carry HSM signatures verifiable against the Commissioner public key in the transparency log.
- **Spoilage chain** - every `SPOILAGE` event must reference a lot barcode; off-sequence spoilage alerts ops and M16 ECC simultaneously.
- **KRA clearance linkage** - `PORT_CLEARED` events without a valid `kra_clearance_ref` are rejected; this prevents undocumented lot entry.

## 8. Kenya statute alignment

- **Elections Act §44A** - prescribed ballot security measures; M39 operationalises the chain-of-custody documentation mandate.
- **Elections (Technology) Regulations 2017 Reg 9** - voting materials must be secured and tracked; M39 satisfies via signed `CustodyEvent` chain.
- **Kenya Revenue Authority Act** - KRA e-clearance integration satisfies customs duty and import documentation requirements for ballot paper sourced offshore.
- **Evidence Act Cap 80** - Merkle-anchored chain of custody is admissible as electronic evidence; M39 produces the proof artefact consumed by M40.
- **Kenya Data Protection Act §26** - no voter PII in ballot chain records; lot-to-station mapping is public-register data, not personal data.

## 9. Honest caveats + external prerequisites

- MES integration requires a formal MoU with the appointed ballot printer and provisioning of an IEBC SVID for the printer's MES system. The adapter at `integrations/ballot-mes/` is written to a documented webhook schema; the exact MES API endpoint is set at printer onboarding.
- KRA customs API integration (`integrations/kra-customs/`) requires a KRA API key provisioned under the IEBC eTrade account. The adapter is ready; the live key is an external procurement dependency.
- RFID scanner devices must be procured with the correct HF RFID standard (ISO/IEC 15693 for paper ballots). The Flutter app targets standard USB-HID RFID readers; proprietary SDKs are not supported.
- Lot-to-ward allocation logic assumes constituency boundary data from M18 (boundary transparency) is settled before `WAREHOUSE_OUT` events are emitted. A boundary dispute that changes ward allocations after lots are dispatched requires a manual override procedure (documented in `docs/runbooks/ballot-chain-reallocation.md`).

## 10. Test coverage + operational runbooks

Tests under `services/inventory-svc/crates/printing/tests/`: integration via `testcontainers` for PG + Kafka + Debezium, RFID scan simulation, reconciliation arithmetic property-based tests via `proptest`, chaos scenarios under `chaos/scenarios/eel-ballot-chain-*.yaml`.

Runbooks under `docs/runbooks/`:

- `inventory-svc.md` - deploy, rollback, MES adapter failure mode.
- `ballot-chain-reallocation.md` - lot reallocation on boundary change.
- `rfid-scanner-offline.md` - fallback to barcode manual entry when RFID fails.
- `station-recon-mismatch.md` - resolution protocol for reconciliation discrepancies.

Related documents: spec at `docs/plans/modules/ballot-printing-chain.md`; capability catalogue at `services/inventory-svc/docs/CAPABILITIES.md`; M40 evidence bundle at `docs/features/election-court-it-interface.md`.
