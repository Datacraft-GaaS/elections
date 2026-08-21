# ADR 0015: Cross-service consistency & event bus

Status: accepted
Date: 2026-08-21

## Context

State propagates across ~100 services via Kafka. Two failure modes must be impossible: dual-write (state and event committed separately, one succeeds and one does not) and silent event tampering. The event bus is also the spine of the transparency log, ClickHouse audit sink, LiveView fan-out, and cross-context projections. Envelope and topic conventions decided once cost nothing; discovered per-service they cost forever. Full derivation in `docs/architecture/decision-log.md` §3 Phases 10-13 and design-log §B.

## Decision

### Transactional outbox is the only path to Kafka

Every service publishes via **transactional outbox + Debezium CDC**. Raw Kafka producers are not exposed to service code; the `iebc-outbox-{rs,py,ex}` shared library is the only public API.

```
Service transaction:
  BEGIN
    INSERT INTO domain_state (...)
    INSERT INTO outbox (event_id, topic, key, payload, headers, signature)
  COMMIT

Debezium:
  reads outbox via logical replication slot
  publishes to Kafka with exactly-once semantics
  marks outbox row published_at

Consumer:
  idempotent by event.id
  updates own state + own outbox transactionally
```

No dual-write hazard. No 2PC. Exactly-once effective delivery when paired with idempotent consumers.

### Envelope: CloudEvents 1.0 + IEBC extensions, JSON only

One encoding: **JSON on the wire with Kafka zstd compression**. Canonical JSON (RFC 8785) is the signing form.

Required fields per event:

```json
{
  "specversion": "1.0",
  "id": "01950f8c-...",                    // UUIDv7
  "type": "ke.iebc.rts.form34a_captured.v1",
  "source": "spiffe://iebc.ke/service/rts-backend",
  "subject": "form-34a/47-0001-001",
  "time": "2027-08-15T15:30:00.123Z",
  "datacontenttype": "application/json",
  "dataschema": "https://schemas.iebc.ke/rts/form34a_captured/v1.json",
  "data": { ... },
  "iebc_stream_id": "01950f8c-...",
  "iebc_seq": 12345,
  "iebc_hlc": "1723731000123.5.42",
  "iebc_prev_hash": "sha256:...",
  "iebc_correlation_id": "01950f8c-...",
  "iebc_causation_id": "01950f8c-...",
  "iebc_producer_key_id": "spiffe-svid:fp",
  "iebc_signature": "ed25519:base64...",
  "iebc_election_id": "2027-general",
  "iebc_data_class": "B"
}
```

Wall-clock `time` legally authoritative (NTP-authenticated). HLC for causal ordering.

### Type & topic naming

- Type: `ke.iebc.<context>.<event_name>.v<major>` (lowercase snake_case).
- Topic: **per aggregate + major version** — `iebc.<context>.<aggregate>.v<major>`. All types for the aggregate's lifecycle share the topic; consumers filter by `type`. Estimated ~500 topics across all modules.

### Partitioning

Partition key = `iebc_stream_id` (aggregate root). Ordering guaranteed per aggregate; parallelism across aggregates.

Partitions by volume tier: **12 (standard) / 32 (medium) / 64 (high volume)**. Never reduce; over-provision by 2x day-1 need.

### Cluster settings — non-negotiable

RF=3, `min.insync.replicas=2`, `unclean.leader.election.enable=false`, `acks=all`, `enable.idempotence=true` on producers.

### Retention — four topic classes

| class | example | hot | cold |
|---|---|---|---|
| Event-source (consensus) | `iebc.rts.*`, `iebc.custody.*`, `iebc.credential.*`, `iebc.declaration.*`, `iebc.ballot_serial.*` | infinite | MinIO via Kafka Tiered Storage (KIP-405) after 30d |
| Operational | `iebc.turnout.*`, `iebc.grievance.*`, `iebc.dispatch.*` | 30d | none |
| Analytics | `iebc.kit.heartbeat.v1`, `iebc.queue.observation.v1`, `iebc.viewer.session.v1` | 7d | none |
| DLQ / tamper | `<topic>.dlq`, `<topic>.tamper` | 90d | none |

Compaction reserved for state topics only (`iebc.config.*.v1`, `iebc.feature_flag.v1`).

### Signing on the bus

Every event on **event-source and credential topics** is Ed25519-signed by the producer with a SPIFFE-issued short-lived workload key (rotated hourly, HSM-anchored chain — see ADR-0005).

- Signature covers canonical JSON (RFC 8785) of envelope minus `iebc_signature`.
- Fingerprint in `iebc_producer_key_id`; verifier fetches key from SPIRE bundle (cached).
- Verification failures never applied; ship to `<topic>.tamper` + immediate page.

### Delivery semantics

| consumer class | semantic |
|---|---|
| Event-sourced projections | at-least-once + idempotent by event.id |
| Transparency log ingest | at-least-once + dedupe by (source, id) |
| Real-time LiveView fan-out | at-most-once acceptable |
| Audit sink to ClickHouse (Phase 2) or Postgres audit | at-least-once + idempotent |
| **Tally aggregation** | **exactly-once (Kafka transactions)** |
| **Result declaration** | **exactly-once** |
| **Credential issuance** | **exactly-once** |

Exactly-once reserved for cases where duplicate application causes legal-scale damage.

### Schema registry

**Karapace** (Aiven Apache 2.0 fork of Confluent SR), deployed per site, backed by `_schemas` topic.

Schemas live in `packages/event-schemas/<context>/<aggregate>/<event>/v<n>.schema.json`, loaded on deploy. CI gates: backward-compatible additions only within a version; breaking change = new version.

### Cross-site mirroring

**MirrorMaker 2** replicates event-source and credential topics primary → mirror (async, RPO < 10s) and primary → escrow (wider budget). Client `bootstrap.servers` = all three sites' broker VIPs; automatic reconnect on primary loss.

### Cluster sizing per site

12 broker nodes + 3 Kraft controller nodes. Broker: 32 cores / 128GB RAM / 4TB NVMe hot + Ceph-backed cold via Tiered Storage. Peak budget 300 MB/s (2x headroom over projected 150 MB/s).

### Replay

- Tiered Storage segments in MinIO, seamless replay via native client.
- Rate-limited replay consumer group with throttled fetch (10 MB/s per partition).
- Snapshot-then-tail: bootstrap from projection snapshot, tail Kafka from snapshot HLC.
- Replay-only cluster for full historical rebuild (RLA audit, forensic).

### Governance

Every new topic: schema PR + topic-config PR + Cilium NetworkPolicy + Kafka ACL + one idempotency test. Deletion: 2 approvers + 30-day tombstone with zero-consumer-activity check.

### Observability

Trace propagation via `iebc_correlation_id` + `iebc_causation_id` woven into OTel spans. Producer metrics (publish rate/error/outbox lag/sign time), consumer metrics (lag per partition Burrow-style, throughput, DLQ rate, dedup hit, tamper arrivals). Alerts: consumer lag >60s = page; `<topic>.tamper` arrival = page; ISR shrink = page.

### What does NOT go on Kafka

Bulk data transfer (MinIO signed URLs), request-response (gRPC/REST), long-running orchestration (service state machines), session state (Valkey), hourly-changing config (K8s ConfigMap / feature flags), public read fan-out (signed static tarballs).

## Consequences

Positive: no dual-write hazard; provable signature chain from producer through log; consistent envelope across ~100 services; replay is bounded and observable; cross-site failover is automatic.

Negative: outbox is one more table per service to maintain; Debezium is one more moving part per T1 cluster; canonical-JSON canonicalisation adds ~200μs per emit; exactly-once semantics reduce tally-topic throughput ~5-10%. All accepted given correctness stakes.

## Alternatives rejected

- Direct Kafka producers in service code: reintroduces dual-write hazard.
- Two encodings (JSON + MessagePack): not worth the codec surface at our volumes given zstd.
- Confluent Schema Registry: Community License (source-available), not clean OSS.
- Apicurio: more features than needed; Karapace simpler.
- 2PC across services: does not scale, adds coupling.
- Exactly-once everywhere: 5-10% throughput cost across ~100 services not worth it where idempotent-at-least-once suffices.

## Related

ADR-0003 (Kafka + Karapace + Debezium), ADR-0005 (producer signing key roots), ADR-0012 (outbox lives in T1 clusters), ADR-0013 (event stores), ADR-0017 (log ingests signed events).

See also `docs/EVENT-CONVENTIONS.md` for the authoring guide for new event types.
