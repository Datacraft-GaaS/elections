# voter-card

Module M53 voter ID card production and collection logistics (currently scope-gated, phase 2).

Manages voter ID card lifecycle: batch creation (from voter register snapshot), collection site assignment, collection record tracking, card pickup confirmation, unclaimed card handling. Integrates with NRB (National Registration Bureau) for card issuance. Tracks card serial numbers for anti-counterfeiting.

Fits into bvr-svc as the card logistics system: batch voters to cards, route to collection sites, track pickup.

## Public API

Exposed modules via lib.rs:
- card_batch: CardBatch, create_batch_from_snapshot(voter_snapshot) -> CardBatchId
- collection: CollectionRecord, record_collection(card_id, voter_id, date, location)
- tracking: CardTracking, get_card_status(card_id) -> CardStatus
- nrb_integration: NRBClient, request_card_issuance(batch)

## Feature flags

scope-gated: Module disabled by default, requires explicit feature flag to enable (phase 2).

## Testing

Unit tests (card batch logic):
```bash
cargo test --lib --features scope-gated
```

Collection tracking tests:
```bash
cargo test collection:: --features scope-gated
```

NRB integration tests (with mock):
```bash
cargo test nrb_integration:: --features scope-gated,test-mocks
```

Integration tests (requires scope-gated):
```bash
cargo nextest run --features scope-gated
```

## Dependencies of note

Workspace:
- bvr-domain: Voter snapshot
- eel-events-rs: Card event emission

External:
- tokio: Async runtime

## Related docs

See services/bvr-svc/docs/voter-card-production.md for card specification.
See services/bvr-svc/docs/nrb-card-issuance.md for NRB integration.
See services/bvr-svc/docs/card-collection-sites.md for collection logistics.
