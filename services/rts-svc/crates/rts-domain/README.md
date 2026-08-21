# rts-domain

Core election result aggregates, event definitions, and TLA+-verified tally logic.

Defines Form34A/B/C aggregate (parsed vote counts, station metadata, timestamp, signature). PublicationSnapshot (tally checkpoint, merkle root, inclusion proof). MerkleRoot (hash chain for audit). TallyEvent (vote tally increment, update, conflict). Implements domain logic: form validation (vote sum vs ballot count), dedup (same station + timestamp), consistency checks.

TLA+ formal models in docs/ verify tally correctness under run-off, tie, and referendum threshold scenarios.

Fits into rts-svc as the domain model: encodes election result semantics and audit requirements.

## Public API

Exposed modules via lib.rs:
- form34: Form34A, Form34B, Form34C, parse_form(bytes) -> Form34Result
- tally_event: TallyEvent, apply_form_to_tally(form, tally) -> UpdatedTally
- publication_snapshot: PublicationSnapshot, merkle_root, tree_size, timestamp
- merkle: MerkleRoot, compute_root(forms) -> RootHash

## Feature flags

None currently.

## Testing

Unit tests (form parsing, tally logic):
```bash
cargo test --lib
```

Property tests (tally consistency):
```bash
cargo test --lib tally_event::
```

TLA+ formal verification (tally correctness):
```bash
cd docs && apalache check tally.tla
```

Integration tests:
```bash
cargo nextest run
```

## Benchmarks

Form parsing latency (if benches/ exists):
```bash
cargo bench parse_form_100k
```

Tally computation for 46k stations:
```bash
cargo bench compute_tally_46k
```

## Dependencies of note

Workspace:
- eel-eventstore-rs: Event persistence
- eel-events-rs: Event schema
- eel-crypto-rs: Merkle root hashing

External:
- serde: Form deserialization
- sha2: Merkle hashing
- proptest: Property testing

## Related docs

See services/rts-svc/docs/form34-spec.md for form structure.
See services/rts-svc/docs/tally-semantics.md for tally algorithm.
See services/rts-svc/docs/merkle-tree-design.md for inclusion proofs.
See docs/formal-verification/tally.tla for TLA+ model.
