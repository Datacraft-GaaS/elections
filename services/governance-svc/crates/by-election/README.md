# eel-governance-by-election

M50 By-election operations: < 21-day notice constraint enforcement, cost tracking, staff register, special election lifecycle management.

## What it does

Manages by-election logistics and timelines: tracks costs (printing, logistics, staff), enforces the 21-day constitutional minimum notice period (Elections Act, statutory provision), builds poll-station staff rosters, and coordinates with inventory-svc for ballot printing and dispatch. Integrates with payroll-svc for temporary staff payments.

Fits in governance-svc as the by-election coordinator. Runs on port 8093 (shared with recall and referendum); emits ByElectionDeclared, StaffRosterFinalized, CostReportCertified events.

## Public API

`eel_governance_by_election::election::ByElection` aggregate: `fn declare(&mut self, date: chrono::DateTime<Utc>) -> Result<()>`, `fn add_staff_record(&mut self, record: StaffRecord) -> Result<()>`, `fn certify_cost_report(&mut self, report: CostReport) -> Result<()>`.

Endpoints:
- `GET /v1/governance/special-elections/{id}` - fetch by-election metadata
- `POST /v1/governance/special-elections/{id}/cost-report` - submit costs
- `GET /v1/governance/special-elections/{id}/staff-roster` - list assigned staff
- `POST /v1/governance/special-elections/{id}/declare-ready` - transition to Ready-to-Vote

Declaration endpoint rejects if notice period < 21 days; emits TimelineViolation event if attempted.

## Feature flags

`#[cfg(feature = "inventory-sync")]` for live inventory-svc integration; tests use local inventory stub.

## Test commands

```bash
cargo test -p eel-governance-by-election --all-features
cargo test -p eel-governance-by-election --lib notice_period_validation
cargo test -p eel-governance-by-election --test integration_by_election -- --include-ignored
```

Verify 21-day constraint via mock datetime injection; test cost aggregation and staff roster validation.

## Bench commands

```bash
cargo bench -p eel-governance-by-election roster_build_1k_stations
```

Building staff roster for 1k stations should complete in < 100ms.

## Dependencies of note

- `chrono`: DateTime arithmetic for 21-day check
- `uuid6`: election and staff record IDs
- `tokio`: inventory-svc async calls
- `serde_json`: cost report parsing

## Related docs

- `docs/api-reference.md#by-election` (endpoint contract)
- Elections Act provision on notice period (cited in constraint validation)
- `migration_004_by_elections.sql` (schema: elections, cost_reports, staff_records)
