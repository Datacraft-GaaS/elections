# ADR 0018: Write-path performance targets

Status: accepted
Date: 2026-08-21

## Context

Election day is estimated at 400-600M write events, peak 30-50k writes/sec sustained for one hour, extreme burst 80-150k/sec for 2-3 hours in the evening RTS submission window. Missing this budget is not a graceful failure, it is a national incident. Performance targets are release gates, not aspirations. See `docs/architecture/decision-log.md` §3 Phase 11 for the first-principles estimate.

## Decision

### SLA

- **Sustain 50k writes/sec per T1 cluster for 4 hours at P99 25ms end-to-end (kit-to-ACK).**
- **5x headroom** in staging load test, sustain 250k writes/sec against a full-fidelity test cluster.
- **Regression gate**: no release may regress P99 write latency by >10% at 5x load. CI-enforced.

### PostgreSQL hot-path tuning per T1 cluster

Non-optional:

```
shared_buffers = 32GB              # 25% of 128GB RAM
effective_cache_size = 96GB
work_mem = 32MB
maintenance_work_mem = 2GB
wal_buffers = 128MB
max_wal_size = 64GB
min_wal_size = 8GB
checkpoint_timeout = 15min
checkpoint_completion_target = 0.9
synchronous_commit = remote_write  # cross-site sync, not remote_apply
commit_delay = 100                 # microseconds
commit_siblings = 10
random_page_cost = 1.1             # NVMe
effective_io_concurrency = 256
autovacuum_vacuum_scale_factor = 0.02
autovacuum_naptime = 10s
huge_pages = on
```

- **Events table partitioned by hour** on election day (by day otherwise); auto-drop from hot tier after 30 days to MinIO.
- **fillfactor 80** on state tables that update in place, HOT updates avoid index bloat.
- **BRIN indexes** on time-series columns (event timestamp), 100x smaller than B-tree.
- **WAL on dedicated NVMe** separate from data volume.
- **PgBouncer transaction mode**, hard cap ~400 server-side connections per cluster.
- **Batch inserts** via multi-row INSERT in the outbox writer, 100-1000 rows per commit.

### Election-day operational mode

Different from steady state:

- Migrations frozen (ADR-0014).
- No admin ad-hoc queries on T1 primaries; all reads on replicas or dedicated analytics store.
- Autovacuum aggressive.
- `statement_timeout` = 2s interactive, 500ms on write path.
- Backups run on hot standby, not primary.
- Circuit breakers (ADR-0009) hair-triggered.
- Alerting sensitivity 10x, page on first anomaly.

### Load test as first-class deliverable

- `loadtest/` in repo, populated week 1.
- **Realistic election-day simulator**: 46k virtual kits, bimodal traffic curve, party-agent attestations, RTS submission burst, viewer fan-out.
- Target loads: 1x, 2x, 5x, 10x projected peak.
- Runs against staging weekly from month 2.
- k6 + Gatling + Rust custom driver for kit-behaviour fidelity.

### Escape hatch

If load tests reveal a T1 cluster cannot hold the SLA vertically, **Citus** (Apache 2.0 PG extension) is the sharding path. Migration procedure documented as a follow-on ADR only if triggered.

## Consequences

Positive: SLA is a number, not a hope; every release is gated by it; scaling headroom is proven, not assumed; escape hatch (Citus) is named but not paid for day 1.

Negative: hardware budget is real, 128GB RAM + 32-64 cores + NVMe RAID + dedicated NVMe WAL per primary × 3 clusters × 3 sites is not cheap. Non-negotiable given the write budget.

## Alternatives rejected

- Citus day 1: adds coordinator, cross-shard semantics, rebalancing, complexity ADR-0012 explicitly rejects until proven necessary.
- Looser SLA (P99 50ms): fails during evening burst window; unacceptable.
- Vertical scale-only forever: ceiling exists; Citus is the release valve.

## Related

ADR-0012 (T1 topology), ADR-0014 (freeze), ADR-0009 (circuit breakers), ADR-0015 (outbox path).
