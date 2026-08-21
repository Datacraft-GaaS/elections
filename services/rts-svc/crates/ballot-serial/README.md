# ballot-serial

Module M63 ballot serial number chain of custody tracking and reconciliation.

Tracks ballot issuance batches and their serial ranges. Records spoilt ballots and deduces used vs issued vs spoilt counts. Computes reconciliation hashes to verify no ballot loss or inflation. Emits events for ballot-serial.assigned and ballot-serial.reconciled.

Fits into rts-svc as the ballot inventory system: tracks physical ballot stock, enables reconciliation audits, feeds into Form 34 validation.

## Public API

Exposed modules via lib.rs:
- serial_assignment: SerialAssignment, assign_range(batch_id, start, end) -> IssuanceReceipt
- spoilt_tracking: SpoiltBallotEvent, record_spoilt(batch_id, count) -> ReconciliationEvent
- reconciliation: Reconciliation, compute_hash(issued, used, spoilt) -> ReconcHash
- audit: AuditLog, log_reconciliation_check(batch_id, result)

## Feature flags

None currently.

## Testing

Unit tests (reconciliation math):
```bash
cargo test --lib
```

Property tests (issued vs used + spoilt):
```bash
cargo test --lib reconciliation::
```

Integration tests:
```bash
cargo nextest run
```

## Dependencies of note

Workspace:
- rts-domain: BallotSerial event integration
- eel-events-rs: Event emission

External:
- sha2: Reconciliation hash computation
- serde: Event serialization

## Related docs

See services/rts-svc/docs/ballot-reconciliation.md for audit procedures.
See services/rts-svc/docs/form34-ballot-validation.md for Form 34 integration.
