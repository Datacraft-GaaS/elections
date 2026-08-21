# portal-svc SLA

Module M11, WP-3.9.

## Availability

| Period | Target | Degraded mode |
|---|---|---|
| Election cycle (E-7 to E+30) | 99.9% | Static CDN snapshots served on origin loss; stale label shown |
| Non-election | 99.0% | Planned maintenance allowed |

## Latency (CDN edge to client)

| Metric | Target |
|---|---|
| p95 read | <= 200 ms |
| p99 read | <= 500 ms |
| Inclusion proof (p95) | <= 30 s end-to-end |

## Throughput

| Scenario | Target |
|---|---|
| Peak concurrent viewers | 5M/hr (AC-M11-01) |
| Bulk download | Redirect to MinIO signed URL; edge caches tarball |

## Cache rules (petition window)

| Period | max-age | s-maxage | Notes |
|---|---|---|---|
| T+0..T+7 | 30 s | 30 s | Aggressive invalidation on new snapshot publish |
| T+8..T+30 | 300 s | 300 s | Relaxed; results immutable post-petition |
| T+31+ | 3600 s | 86400 s | Near-static; archive mode |

## RTO

- Origin failover: <= 30 s (FR-M11-019).
- CDN cache continues serving signed snapshots during origin downtime.

## SLO burn-rate alerts

Defined in platform/observability/portal-svc/slo-alerts.yaml (to be created in WP-4):
- p99 read latency > 500 ms for 5 min.
- Snapshot staleness > 60 s.
- Origin availability < 99.9%.
