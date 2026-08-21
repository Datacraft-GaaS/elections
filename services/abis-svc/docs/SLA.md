# abis-svc SLA

References: FR-M03-031, FR-M03-032, FR-M03-033, FR-M03-034. Gate G3.

## Availability

| Target | 99.99% uptime per calendar month |
|--------|----------------------------------|
| Excluded | Planned maintenance windows (max 30 min/month, announced 48h in advance) |
| Measurement | Synthetic probes from all active sites every 30 seconds |

## Latency

1:N match (deduplication):
- p95 <= 5 seconds end-to-end (request accepted to result available)
- p99 <= 8 seconds

1:1 match (identity verification at polling station):
- p99 <= 3 seconds (synchronous path; inline in BVI flow)

These targets cover the full pipeline: API ingestion, Kafka transit, matcher scoring, result write, and event emission.

## Adjudication

| Metric | Target |
|--------|--------|
| First response (case assigned to investigator) | <= 4 hours |
| Full resolution (decision recorded) | <= 24 hours |
| SLA breach alert | Fired at T-2h before deadline |

SLA deadline is stored per case in `sla_deadline_at`. A monitoring job fires a `drift_alert.v1` equivalent when cases approach breach.

## Throughput

| Metric | Target |
|--------|--------|
| Batch dedup throughput | >= 10,000 enrolments/hour per site |
| Concurrent 1:1 match ops | >= 100/second at primary site |

## Recovery

| Metric | Target |
|--------|--------|
| RTO (Recovery Time Objective) | <= 4 hours |
| RPO (Recovery Point Objective) | <= 15 minutes |

RPO is met by PostgreSQL streaming replication to the hot standby with WAL archival to object storage. The outbox ensures no events are lost during a failover window; the relay replays from the last published offset.

## Exclusions

SLA does not cover:
- abis-vendor SDK unavailability caused by the third-party vendor
- Enclave node pool exhaustion during elections (covered by capacity planning SLA separately)
- Network partitions between sites classified as force majeure
