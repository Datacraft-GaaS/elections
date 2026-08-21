# inspection

Voter list public inspection service for M64 module per Elections Act §6A. Manages RollSnapshot (frozen voter register), Objection (data disputes), and CorrectionClaim (voter updates).

Freezes inspection roll at registration closure, publishes it to public portal, handles citizen objections and correction workflows. Fits into portal-svc as the voter roll transparency layer.

## What it does

Maintains frozen voter roll snapshots published for public inspection. Ingests objections (address, name, data errors) and correction claims (evidence links). Tracks objection lifecycle (filed, acknowledged, assigned, resolved). Computes inspection completion metrics and audit reports.

## Public API

Exposed modules via lib.rs:
- roll: RollSnapshot, freeze_roll() -> Snapshot
- objection: Objection, file_objection(data) -> ObjectionId
- correction: CorrectionClaim, submit_correction(data) -> ClaimId
- audit: generate_inspection_report() -> Report

## Feature flags

None currently.

## Testing

Unit tests (roll freezing, objection lifecycle):
```bash
cargo test --lib
```

Objection submission correctness:
```bash
cargo test objection::
```

Audit report generation:
```bash
cargo test audit::
```

Integration tests:
```bash
cargo nextest run
```

## Benchmarks

Roll snapshot generation (100k voters):
```bash
cargo bench freeze_roll_100k_voters
```

## Dependencies of note

Workspace:
- eel-events-rs: Objection and correction events
- eel-pii-rs: PII field masking for public roll

External:
- uuid: Objection tracking IDs
- chrono: Objection timestamp tracking

## Related docs

See services/portal-svc/docs/section-6a-inspection.md for §6A workflow.
See docs/runbooks/inspection-roll-lock-failure.md for troubleshooting.
