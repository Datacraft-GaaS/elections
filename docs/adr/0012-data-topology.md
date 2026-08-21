# ADR 0012: Data topology

Status: accepted
Date: 2026-08-21

## Context

Election-day write budget estimated at 400-600M events, peak 30-50k writes/sec sustained for one hour, extreme burst 80-150k/sec during the evening RTS submission window. 69+ services need PostgreSQL access. Data must be isolated by trust boundary while ops burden must be kept manageable. See `docs/architecture/decision-log.md` §3 Phase 10-13.

## Decision

**Three T1 (Tier 1) PostgreSQL clusters per site, grouped by trust boundary and write intensity.** Times three sites = 9 cluster instances, template-managed by one Patroni + pgBackRest + Debezium playbook set.

| cluster | contents | topology |
|---|---|---|
| **T1-voter** | Turnout/BVI events, register mutations (M02), chain-of-custody (M20), ballot serial chain (M63) | Single primary, hourly-partitioned hot tables, sync replica within site, sync to mirror, async to escrow |
| **T1-consensus** | Results (M08/M09/M10 RTS 34A/B), tally, declaration (M51), credential issuance, transparency-log super-root | Single primary, HSM-hot, sync replicas |
| **T1-audit** | RLA audit events (M21), cross-service audit trail, per-domain transparency log leaves | Single primary, async replicas OK (audit tolerates seconds of lag) |

**No Citus day 1.** Modern PG 17 on serious hardware (64-128 cores, 512GB-1TB RAM, NVMe RAID, dedicated NVMe for WAL) with declarative hourly partitioning + batched inserts + async commit + PgBouncer transaction mode holds 50-100k writes/sec (Instagram, Discord, Notion at this scale on vanilla PG). Citus adds coordinator layer, cross-shard transaction semantics, extension coupling, rebalancing operations — each a class of incident that cannot happen without it. Turn on Citus only if 5x-load tests prove necessity.

**T2 (operational transactional)**: shared PostgreSQL clusters, schema-per-service, row-level security across schemas. Candidate registration (M12), accreditation (M42/M43), CMS (M37), dispatch, inventory operational rows — CRUD-heavy, not consensus-critical.

**T3 (analytics)**: PostgreSQL read replicas + hourly partitions + BRIN indexes + materialised views day 1. ClickHouse deferred.

**T4 (search)**: single OpenSearch cluster for the four search-heavy modules (M07, M18, M22, M52).

**T5 (cache / hot state)**: Valkey per site.

**Standing rules**:
- No cross-schema joins. Services own their data.
- No shared writes. Two services never write to the same table.
- Row-level security enforced for multi-tenant tables (observer, party agent, media, county coordinator).
- PgBouncer transaction mode in front of every cluster; no raw connection pools in services.
- Read replicas per site for T1; shared read pool for T2.
- HA: Patroni + etcd on every PG cluster; failover drills monthly.
- Backups: pgBackRest with WAL to MinIO; full weekly, differential daily, WAL continuous; PITR to any point in 30 days; monthly restore drill.

## Consequences

Positive: three T1 clusters × three sites is manageable; failure blast-radius bounded to one trust boundary; single-primary topology is boring, understood, defensible. Analytics on PG buys time before ClickHouse ops cost lands.

Negative: an incident affecting T1-voter takes down turnout + register + custody together — voter-cluster RTO becomes the load-bearing operational number. If load tests reveal PG cannot hold turnout peak, Citus is the escape hatch (migration path documented as a follow-on ADR when needed).

## Alternatives rejected

- One DB per service (69 clusters): explodes ops.
- Citus day 1: adds complexity we have not proven necessary.
- One shared DB: drowns under contention.
- CockroachDB: BSL.

## Related

ADR-0003 (data infra), ADR-0013 (event-sourcing scope determines which of these are event-source vs state-based), ADR-0014 (schema evolution), ADR-0015 (outbox tables live here), ADR-0018 (write-path performance SLA).
