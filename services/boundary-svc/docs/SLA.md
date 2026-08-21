# boundary-svc SLA

## HTTP API (public)

| Metric | Target |
|--------|--------|
| p95 latency | < 100 ms |
| p99 latency | < 300 ms |
| Availability | 99.9% monthly |
| Bulk GeoJSON tarball freshness | <= 15 min (worker cadence) |

## gRPC Geofence.Check (internal)

| Metric | Target |
|--------|--------|
| p99 latency | < 20 ms local |
| Polygon cache staleness | <= 15 min (worker refresh) |

## Error budget

Monthly downtime budget at 99.9%: ~44 min.

## Measurement

Latency measured at the service ingress with OpenTelemetry histograms exported to Prometheus. Geofence bench: `cargo bench -p boundary-geofence`.
