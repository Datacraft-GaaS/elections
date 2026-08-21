# eel-governance-rla

M21 Risk-Limiting Audit tool: SHANGRLA-based statistical ballot sampling, NIST Randomness Beacon integration, offline Tauri desktop app with local verdict queueing and sync-on-reconnection.

## What it does

Runs RLA audits per US election infrastructure best practices (audits ballots to statistical assurance that reported outcomes are correct). Draws sample sizes via Hypergeometric distribution, feeds entropy to offline audit laptops running the Tauri rla-desktop app, records verdicts, and syncs results back. Integrates NIST Randomness Beacon as audit randomness source; falls back to cached seed on network failure.

Fits in governance-svc as the primary statistical verification engine post-election. Runs independently on port 8091; aggregates results from county-level RLA teams and produces pass/fail verdict per constituency.

## Public API

`eel_governance_rla::audit_run::AuditRun` aggregate: `fn draw_sample(&mut self, seed: [u8; 32]) -> Result<Sample>`, `fn record_verdict(&mut self, sample_id: uuid::Uuid, outcome: Outcome) -> Result<()>`, `fn is_recount_triggered(&self) -> bool`.

Endpoints:
- `POST /v1/governance/rla/runs` - initiate run (manifest JSON)
- `GET /v1/governance/rla/runs/{run_id}` - status, sample count
- `POST /v1/governance/rla/runs/{run_id}/samples` - draw batch
- `GET /v1/governance/rla/runs/{run_id}/report` - final verdict

HTTP middleware handles auth + observability. Database queries use axum extractors.

## Feature flags

`#[cfg(feature = "nist-beacon")]` for live NIST API integration; off in offline tests.

`#[cfg(test)]` tests use `RngSeeder::fixed()` for deterministic sampling.

## Test commands

```bash
cargo test -p eel-governance-rla --all-features
cargo test -p eel-governance-rla --lib sample_distribution
cargo test -p eel-governance-rla --test integration_rla -- --include-ignored
```

Tags: `#[test]` for unit, `#[ignore]` for long-running (30s+).

## Bench commands

```bash
cargo bench -p eel-governance-rla sample_draw_10k
cargo bench -p eel-governance-rla hypergeometric_compute
```

SHANGRLA computation is O(sample_size); benchmarks verify < 100ms for 5k samples.

## Dependencies of note

- `sha2`: NIST beacon digest verification
- `statrs`: hypergeometric distribution CDF
- `uuid6`: Uuid7 for sample IDs
- `tokio`: async retry logic (beacon timeout fallback)
- `serde_json`: ballot manifest parsing

## Related docs

- `docs/tla/rla-sampling.tla` (TLA+ model for sample draw correctness)
- `docs/api-reference.md#rla` (endpoint contract)
- `migration_001_audit_runs.sql` (schema: audit_runs, samples, verdicts tables)
