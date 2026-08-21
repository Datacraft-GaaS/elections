# eel-governance-referendum

M49 Referendum mechanics: county-level result aggregation, 50%+1 national threshold logic, majority-of-counties tie-breaker, TLA+ verified.

## What it does

Manages referendum result certification: ingests county tallies (Form 34C), applies national threshold rule (50%+1 vote share + majority of counties), handles ties (requires electoral commission public announcement), and publishes certified result. All logic formally verified in TLA+ model (`docs/tla/referendum-tiebreak.tla`).

Fits in governance-svc as the referendum aggregation engine. Runs on port 8093 (shared with recall); integrates with transparency-svc to certify results via Merkle log.

## Public API

`eel_governance_referendum::referendum::Referendum` aggregate: `fn add_county_result(&mut self, county_id: str, yes_count: u64, no_count: u64) -> Result<()>`, `fn compute_national_result(&self) -> NationalResult`, `fn is_tie(&self) -> bool`.

Endpoints:
- `GET /v1/governance/referendum/{id}` - fetch metadata
- `POST /v1/governance/referendum/{id}/county-results` - submit tally
- `GET /v1/governance/referendum/{id}/result` - fetch certified result (immutable once published)
- `POST /v1/governance/referendum/{id}/declare-tie` - escalate to electoral commission

Tie-breaker logic requires electoral commission signature before state transitions to Certified.

## Feature flags

`#[cfg(feature = "transparency-log")]` for live Merkle tree signing; tests stub with in-memory signatures.

## Test commands

```bash
cargo test -p eel-governance-referendum --all-features
cargo test -p eel-governance-referendum --lib national_threshold
cargo test -p eel-governance-referendum --test integration_referendum -- --include-ignored
```

Property-based tests (proptest) verify threshold and tie logic on randomized county counts.

## Bench commands

```bash
cargo bench -p eel-governance-referendum aggregate_47_counties
```

Aggregation of 47 county results (Kenya's county count) should complete in < 50ms.

## Dependencies of note

- `uuid6`: referendum IDs
- `proptest`: property-based testing of threshold logic
- `tokio`: transparency-svc async calls
- `serde_json`: county result parsing

## Related docs

- `docs/tla/referendum-tiebreak.tla` (formal model, run `apalache check` to verify)
- `docs/api-reference.md#referendum` (endpoint contract)
- `migration_003_referendums.sql` (schema: referendums, county_results, tiebreaker_declarations)
