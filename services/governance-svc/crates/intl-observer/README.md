# eel-governance-intl-observer

M57 International observer coordination: accreditation tracking, briefing distribution, observation report collection, joint statements.

## What it does

Manages international election observer missions (UN, AU, ECOWAS, Carter Center, etc.): tracks accreditations, issues briefing packs (election law, IT infrastructure, security), collects observation reports from individual observers, aggregates them into mission-level summaries, and publishes joint statements cosigned by mission leads. Integrates with training-svc (accreditation-portal) and transparency-svc (joint statement archival). Uses workspace isolation: each mission has its own PostgreSQL schema to prevent cross-mission data leakage.

Fits in governance-svc as the international relations layer. Runs on port 8093 (shared service); emits governance.observer.mission.briefed, governance.observer.report.submitted events.

## Public API

`eel_governance_intl_observer::mission::ObserverMission` aggregate: `fn register_observers(&mut self, roster: ObserverRoster) -> Result<()>`, `fn distribute_briefing(&mut self, briefing: Briefing) -> Result<()>`, `fn receive_report(&mut self, report: ObservationReport) -> Result<()>`, `fn co_sign_statement(&mut self, lead_id: str, signature: Signature) -> Result<()>`.

Endpoints:
- `GET /v1/governance/international-observers/{mission_id}` - fetch mission metadata
- `POST /v1/governance/international-observers/{mission_id}/briefing` - send briefing
- `GET /v1/governance/international-observers/{mission_id}/briefing` - download briefing
- `POST /v1/governance/international-observers/{mission_id}/reports` - submit observation
- `GET /v1/governance/international-observers/{mission_id}/joint-statement` - fetch final statement

Workspace isolation via `X-Mission-Id` header (validated against JWT mission_id claim).

## Feature flags

`#[cfg(feature = "workspace-isolation")]` for separate schema per mission; tests use logical isolation (same schema, mission_id check).

## Test commands

```bash
cargo test -p eel-governance-intl-observer --all-features
cargo test -p eel-governance-intl-observer --lib workspace_isolation
cargo test -p eel-governance-intl-observer --test integration_intl_observer -- --include-ignored
```

Verify cross-mission data separation; test report aggregation and co-signature workflows.

## Bench commands

```bash
cargo bench -p eel-governance-intl-observer aggregate_500_reports
```

Aggregating 500 observation reports into mission summary should complete in < 200ms.

## Dependencies of note

- `uuid6`: mission and observer IDs
- `tokio`: async report ingestion
- `serde_json`: briefing and report parsing
- `chrono`: mission timeline tracking

## Related docs

- `docs/api-reference.md#intl-observer` (mission lifecycle)
- `docs/architecture.md#workspace-isolation` (schema segregation design)
- `migration_010_intl_observer.sql` (schema: missions, observers, briefings, reports, joint_statements)
