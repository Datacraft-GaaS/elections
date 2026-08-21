# rts-tally

TLA+-formally-verified tally aggregation engine.

Implements referendum threshold logic (50%+1 national, majority per county), run-off handling (top 2 candidates if no majority), tie resolution (return to IEBC for discretion). Formally verified via TLA+/Apalache to ensure correctness under all candidate/voter scenarios. Computes valid election result or identifies ambiguous case requiring manual ECC intervention.

Fits into rts-svc as the tally computation core: takes vote counts, applies rules, returns result or escalation flag.

## Public API

Exposed modules via lib.rs:
- tally_engine: TallyEngine, compute_result(votes_per_candidate) -> ElectionResult
- referendum: ReferendumTally, check_referendum_threshold(yes, no) -> ApprovalResult
- runoff: RunoffDetection, detect_runoff_required(vote_counts) -> RunoffCandidates
- tie_handler: TieHandler, detect_tie(sorted_votes) -> TieCase

## Feature flags

None currently.

## Testing

Unit tests (threshold logic):
```bash
cargo test --lib
```

Property tests (edge cases):
```bash
cargo test --lib referendum::
cargo test --lib runoff::
```

TLA+ formal verification (correctness proof):
```bash
cd docs && apalache check tally_logic.tla
```

Integration tests:
```bash
cargo nextest run
```

## Benchmarks

Tally computation for 46k stations (if benches/ exists):
```bash
cargo bench tally_46k_stations
```

Referendum threshold check (1M vote counts):
```bash
cargo bench referendum_1m_votes
```

## Dependencies of note

Workspace:
- rts-domain: Vote count types

External:
- proptest: Property testing for edge cases

## Related docs

See services/rts-svc/docs/tally-algorithm.md for detailed algorithm.
See docs/formal-verification/tally_logic.tla for TLA+ model and proofs.
See services/rts-svc/docs/referendum-rules.md for threshold rules.
See services/rts-svc/docs/runoff-scenarios.md for run-off handling.
