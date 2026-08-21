# custody

Event-sourced asset custody ledger for inventory tracking. Manages Asset, CustodyEvent (HSM-signed), Seal, and Consumable aggregates. Provides full custody chain reconstruction and current asset state queries.

Core of inventory-svc: tracks all physical asset handoffs with cryptographic signatures for non-repudiation. Emits custody.transferred, kit.dispatched, and asset.received events for downstream consumption and audit verification.

## What it does

Maintains event-sourced ledger of asset ownership changes. Each CustodyEvent is HSM-signed to prevent tampering. Supports asset types: KIEMS kits, ballot paper, seals, consumables. Reconstructs full custody chain from events. Computes current asset state (location, status, custodian). Detects unverified events (audit trail gaps).

## Public API

Exposed modules via lib.rs:
- asset: Asset, AssetId, asset_type() -> Type
- custody: CustodyEvent, record_event(from, to, asset) -> EventId
- seal: Seal, apply_seal(asset) -> SealId
- ledger: reconstruct_chain(asset_id) -> Chain
- hsm: sign_event(event, hsm_key) -> Signature

## Feature flags

None currently.

## Testing

Unit tests (custody logic, event sourcing):
```bash
cargo test --lib
```

HSM signature tests:
```bash
cargo test hsm::sign_and_verify
```

Custody chain reconstruction:
```bash
cargo test ledger::reconstruct_chain
```

Integration tests (event sourcing):
```bash
cargo nextest run
```

## Benchmarks

Event recording latency:
```bash
cargo bench record_custody_event
```

## Dependencies of note

Workspace:
- eel-events-rs: Event emission
- eel-observability-rs: Ledger metrics

External:
- uuid: Asset and event IDs
- chrono: Event timestamps
- pkcs11: HSM signing

## Related docs

See services/inventory-svc/docs/custody-model.md for ledger design.
See services/inventory-svc/docs/hsm-integration.md for signing procedures.
See docs/runbooks/custody-event-backlog.md for troubleshooting.
