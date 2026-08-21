# observer_workspace (Phoenix app)

Isolated Phoenix LiveView workspace for accredited international observer missions.

## Module

M57 (intl-observer-coord) | governance-svc | WP-5.9

## Purpose

Tenant-isolated Phoenix LiveView for accredited observer mission organisations (AU, EU,
Carter Center, Commonwealth, etc.). Each mission gets a scoped workspace instance.
Supports briefing calendar access, observation report submission, joint statement
drafting (collaborative), and export of the final report package. Workspace isolation
is enforced at Phoenix PubSub scope and database row-level security.

## LiveViews

| Module | Description |
|---|---|
| `MissionBriefingLive` | Commission briefing schedule and materials for the mission |
| `ObservationReportLive` | Observation report entry by polling station or region |
| `JointStatementLive` | Collaborative joint statement editor with version history |
| `BriefingCalendarLive` | Calendar view of scheduled briefings and events |

## Broadway Consumers

None.

## Oban Queues

| Queue | Workers | Description |
|---|---|---|
| `:report_publish` | `ReportPublishWorker` | Export and sign final observation report PDF |

## Rustler NIF Usage

None.

## Mix Commands

```bash
cd services/governance-svc/ex/observer_workspace
mix deps.get
mix ecto.create && mix ecto.migrate
mix phx.server
MIX_ENV=test mix test
```

## Configuration / Env Variables

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL DSN |
| `SECRET_KEY_BASE` | Phoenix secret key |
| `OBSERVER_MISSION_ID` | Mission identifier (injected per deployment) |
| `GOVERNANCE_API_URL` | governance-svc Rust API |
| `TENANT_ISOLATION_KEY` | Row-level security scope key |

## WCAG

2.2 AA. All collaborative editing accessible via keyboard and screen reader.

## Deployment Notes

- Kubernetes: governance-svc (one deployment per accredited mission, isolated namespace)
- K8s manifest: services/governance-svc/deploy/base/observer-workspace-deployment.yaml
- Mission namespaces provisioned 30 days before election day

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M57)
- governance-svc/crates/intl-observer
- training-svc/django/accreditation (observer accreditation source)
