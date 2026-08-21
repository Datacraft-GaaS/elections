# diaspora-logistics

Diaspora ballot pouch logistics management for M65 module. Coordinates shipping, customs clearance, and embassy delivery for diaspora ballots.

Manages BallotContainer, CustodyEvent, PreClearanceManifest aggregates. Integrates with airlines (tracking), customs (MFA/KRA), and embassies for end-to-end pouch accountability. Fits into inventory-svc as the diaspora ballot supply chain layer.

## What it does

Tracks diaspora ballot pouch lifecycle: print and seal, airline handoff, customs pre-clearance, in-flight tracking, embassy receipt, station delivery. Emits custody events for each handoff. Coordinates with airline APIs for tracking and customs authorities for clearance status. Flags holds or delays exceeding SLA.

## Public API

Exposed modules via lib.rs:
- container: BallotContainer, PoachId, seal_pouch(ballots) -> ContainerId
- shipment: ShipmentEvent, create_shipment(containers, airline) -> ShipmentId
- customs: PreClearanceManifest, file_manifest(shipment) -> ManifestId
- embassy: receive_at_embassy(shipment, embassy) -> ReceiptId
- tracking: get_tracking_status(shipment_id) -> Status

## Feature flags

None currently.

## Testing

Unit tests (pouch lifecycle, custody events):
```bash
cargo test --lib
```

Customs manifest generation:
```bash
cargo test customs::
```

Shipment tracking status:
```bash
cargo test tracking::
```

Integration tests (airline and customs mock APIs):
```bash
cargo nextest run
```

## Benchmarks

Manifest generation (5000 pouches):
```bash
cargo bench generate_manifest_5k_pouches
```

## Dependencies of note

Workspace:
- custody: CustodyEvent recording
- eel-events-rs: Shipment events
- eel-observability-rs: Logistics metrics

External:
- reqwest: Airline and customs API clients
- uuid: Pouch and shipment tracking IDs
- chrono: Timeline tracking

## Related docs

See services/inventory-svc/docs/diaspora-logistics-flows.md for airline/customs integration.
See services/inventory-svc/docs/customs-manifest-format.md for KRA/MFA requirements.
See docs/runbooks/diaspora-customs-hold.md for SLA violations.
