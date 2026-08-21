# boundary-svc

## Purpose

Manages electoral boundary data, publishes shapefile-backed constituency and ward polygons, tracks boundary changes through commissions (NLC/KNBS), and exposes fairness metrics. It is the authoritative source for geospatial context used by kit provisioning, turnout analysis, and the public mapping API.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M18 | boundary-transparency | Boundary dataset, change tracking, fairness metrics, public geofence API |

## Key Aggregates and State Stores

- `Boundary` (PostgreSQL + PostGIS): constituency and ward polygons with version history.
- `BoundaryChange` (PostgreSQL, append-only): NLC/KNBS gazette-sourced change events.
- `FairnessMetric` (PostgreSQL): computed equity and deviation indices per boundary.
- `GeofencePolygon` (PostGIS): kit geofencing lookup table.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | bvr-svc | Voter register for fairness-metric computation |
| External | NLC (National Land Commission) | Authoritative shapefile source |
| External | KNBS | Population figures for equity computation |
| Downstream | inventory-svc | Kit geofencing polygons |
| Downstream | portal-svc | Public boundary maps |
| Downstream | ecc-svc | Boundary alerts for operational planning |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.boundary.boundary.changed` | `eel.boundary` | Boundary polygon updated via gazette |
| `eel.boundary.fairness_metric.computed` | `eel.boundary` | New fairness metric published |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.bvr.register.snapshot.published` | bvr-svc |

## Primary APIs

- `packages/openapi-contracts/boundary/openapi.yaml` - boundary query, fairness metrics, geofence lookup

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Boundary query P99 | less than 100 ms |
| Availability | 99.5% |
| RTO | 15 min |
| RPO | 1 h (daily shapefile snapshot) |

## Degradation Modes

- If NLC feed is unavailable, last known shapefile remains active; a staleness alert fires after 24 h.
- Fairness metric computation defers without blocking read path.
- PostGIS read replica handles public map queries if primary is under load.

## Runbook Links

- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
