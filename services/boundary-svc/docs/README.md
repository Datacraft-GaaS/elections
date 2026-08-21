# boundary-svc

Module M18: Boundary Transparency.

Provides public read access to Kenya's electoral boundaries (counties, constituencies, wards, polling stations) and admin-gated tools for boundary change management and geofence polygon publishing.

## SPIFFE identity

`spiffe://eel.local/service/boundary-svc`

## Ports

| Port | Protocol | Purpose |
|------|----------|---------|
| 8080 | HTTP/1.1 | Public + admin REST API |
| 9090 | gRPC     | Internal `Geofence.Check` (kit-side, bvi-svc) |

## SLA

See `SLA.md`. Public endpoints: p95 < 100 ms, p99 < 300 ms, availability 99.9%.
Geofence.Check: p99 < 20 ms local.

## Data classification

All boundary geometry is class C (public). See `data_dictionary.yaml` for per-column classification.

## Worker cadence

boundary-worker runs three jobs every 15 minutes:
- Shapefile ingest from MinIO `eel-shapefiles`.
- Fairness metric recomputation (Polsby-Popper compactness, population equality, VRA-analog).
- Signed GeoJSON tarball production to MinIO `eel-boundary-geojson`.

## Unsigned polygon guard

Any geofence polygon with an empty `signature` field is rejected at ingest (HTTP 422, domain error). This is enforced in `boundary_domain::types::GeofencePolygon::validate_signature` and the `POST /api/v1/geofence/upload` handler.

## Events

See `EVENTS.md`.

## Threat model

See `THREAT-MODEL.md`.
