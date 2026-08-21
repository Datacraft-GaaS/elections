# printing

Ballot printing chain management for M39 module. Coordinates printer MES (Manufacturing Execution System) integration, lot certification, RFID tagging, and ballot custody from print through station delivery.

Manages PrintOrder, LotData, CustodyEvent, StationReconciliation aggregates. Fits into inventory-svc as the ballot supply chain layer: connects printer production to physical ballot custody and eventual reconciliation against Form34 ballot counts.

## What it does

Integrates with printer MES to track ballot production runs. Records lot data: ballot style, quantity, print date. Coordinates RFID tagging of sealed ballot boxes. Certifies lot ready for dispatch. Emits custody events as ballots move through warehouse and into field. Reconciles received ballots against Form34 counts at each station.

## Public API

Exposed modules via lib.rs:
- order: PrintOrder, OrderId, create_order(styles, quantities) -> OrderId
- lot: LotData, certify_lot(order_id, print_date) -> LotId
- reconciliation: StationReconciliation, reconcile_ballots(station_id, form34_count) -> ReconciliationResult
- mes_integration: fetch_production_run(order_id) -> ProductionData
- rfid: assign_rfid_tags(lot_id, box_count) -> RfidAssignments

## Feature flags

None currently.

## Testing

Unit tests (order lifecycle, certification):
```bash
cargo test --lib
```

MES integration (mock printer):
```bash
cargo test mes_integration::
```

Ballot reconciliation logic:
```bash
cargo test reconciliation::
```

RFID tagging coordination:
```bash
cargo test rfid::
```

Integration tests:
```bash
cargo nextest run
```

## Benchmarks

Lot certification latency:
```bash
cargo bench certify_lot_10k_ballots
```

## Dependencies of note

Workspace:
- custody: Custody event recording
- eel-events-rs: Production and reconciliation events
- eel-observability-rs: Production metrics

External:
- reqwest: MES API client
- uuid: Order and lot tracking IDs
- chrono: Production timeline

## Related docs

See services/inventory-svc/docs/printer-mes-integration.md for printer API format.
See services/inventory-svc/docs/ballot-reconciliation.md for Form34 matching logic.
See docs/runbooks/printing-reconciliation-discrepancy.md for troubleshooting.
