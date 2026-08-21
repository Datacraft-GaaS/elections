# ADR 0003: Data infrastructure

Status: accepted
Date: 2026-08-21

## Context

We need OLTP, analytics, search, cache, message bus, and object storage across 69+ services under Kenya-sovereign hosting, 15-year audit retention, air-gapped build path, and strict OSS-hygiene (no SSPL, BSL, Confluent Community, Source-Available). See `docs/architecture/decision-log.md` §3 Phase 3 and §4.

## Decision

- **OLTP**: PostgreSQL 17 with PostGIS (M18 boundaries, M19 queue geo), pgvector (M17, M28 embeddings), pg_partman (partitioning), pgcrypto, pgaudit. Patroni HA + etcd, pgBackRest to MinIO, PgBouncer transaction mode. Topology in ADR-0012.
- **Analytics day 1**: PostgreSQL read replicas + hourly partitions + BRIN indexes + materialised views. **ClickHouse deferred** to Phase 2; a module ADR must justify its introduction.
- **Search**: **OpenSearch** (Apache 2.0 fork of Elasticsearch) for the four modules that need full-text + fuzzy + geo (M07 register inspection, M18 boundary search, M22 candidate search, M52 gazette search). All other modules use Postgres full-text (pg_trgm + tsvector).
- **Cache / hot state**: **Valkey 8** (Linux Foundation BSD-3 fork of Redis), sharded per site. Redis 7.4+ rejected (BSL).
- **Message bus**: **Apache Kafka 3.7+ in Kraft mode** (no ZooKeeper), RF=3, `min.insync.replicas=2`, `unclean.leader.election.enable=false`, `acks=all`. Debezium for CDC/outbox. Envelope + conventions in ADR-0015. Redpanda Community rejected (Source-Available BSL).
- **Schema registry**: **Karapace** (Aiven Apache 2.0 fork of Confluent SR).
- **Object store**: **MinIO** (S3-compatible, on-prem). Verify licence fit before scale.
- **CDC**: Debezium (Apache 2.0) reads outbox tables → publishes to Kafka.

## Consequences

Positive: every choice is Apache 2.0 or equivalent; every dependency is mirrorable to internal Nexus. Operational familiarity for MinIO, Valkey, Postgres, Kafka is high. Deferring ClickHouse and scoping OpenSearch trims ~50% of the day-1 stateful surface area versus the original proposal.

Negative: Postgres + materialised views for analytics has a scale ceiling; if a module hits it, that module writes an ADR to bring ClickHouse in for its domain. OpenSearch cluster is one more thing to secure, but limited to four modules.

## Alternatives rejected

- Elasticsearch (SSPL), Redis 7.4+ (BSL), HashiCorp Vault (BSL), Redpanda Community (BSL), Confluent Schema Registry (Community License). All out on OSS-hygiene grounds even where they would be operationally convenient.
- ClickHouse day 1: over-engineered for our year-1 analytics loads; keeps ops surface leaner.
- MongoDB (SSPL) and CockroachDB (BSL): rejected upstream on licence grounds.

## Related

ADR-0004 (platform hosts these), ADR-0006 (Nix builds all of them), ADR-0012 (Postgres topology), ADR-0015 (Kafka conventions), ADR-0017 (transparency log storage).
