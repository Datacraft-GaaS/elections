# Event conventions

Authoring guide for anyone adding a new event type or topic. Canonical reference; every module doc points here rather than restating rules.

The full decision text lives in **ADR-0015**. This document is the operational cheat-sheet.

---

## Adding a new event type — checklist

Every new event type requires all of the following in the same PR:

- [ ] JSON Schema written at `packages/event-schemas/<context>/<aggregate>/<event_name>/v<n>.schema.json`.
- [ ] Type name matches `ke.iebc.<context>.<event_name>.v<major>` (lowercase snake_case).
- [ ] Topic exists at `iebc.<context>.<aggregate>.v<major>` (or PR to create it in `platform/kafka/topics/`).
- [ ] Producer service publishes via `iebc-outbox-{rs,py,ex}` shared library — never a raw Kafka producer.
- [ ] Ed25519 signature produced by SPIFFE-issued workload key (automatic if using the shared library).
- [ ] Consumer test proves idempotency by `event.id`.
- [ ] If event is on an event-source or credential topic: signed-verification test on the consumer path.
- [ ] Data-class of payload declared in event schema (A/B/C/D/E per ADR-0016).
- [ ] Owner + slack channel in schema metadata.
- [ ] Change Log entry in the owning module's `docs/EVENTS.md`.

CI enforces: schema validity, `type` ↔ topic mapping, backward compatibility within a version, data-class presence.

---

## Envelope reference

Every event carries CloudEvents 1.0 required fields plus IEBC extensions. The shared library populates all of these — service code sets `data` and (optionally) `subject`, `iebc_correlation_id`, `iebc_causation_id`.

```json
{
  "specversion": "1.0",
  "id": "01950f8c-...",                    // UUIDv7, globally unique
  "type": "ke.iebc.rts.form34a_captured.v1",
  "source": "spiffe://iebc.ke/service/rts-backend",
  "subject": "form-34a/47-0001-001",       // aggregate identifier
  "time": "2027-08-15T15:30:00.123Z",      // NTP-authenticated wall clock
  "datacontenttype": "application/json",
  "dataschema": "https://schemas.iebc.ke/rts/form34a_captured/v1.json",
  "data": { ... payload matching the schema ... },
  "iebc_stream_id": "01950f8c-...",        // aggregate root UUIDv7
  "iebc_seq": 12345,                       // monotonic within stream
  "iebc_hlc": "1723731000123.5.42",        // hybrid logical clock
  "iebc_prev_hash": "sha256:...",          // chain within stream
  "iebc_correlation_id": "01950f8c-...",   // trace root
  "iebc_causation_id": "01950f8c-...",     // originating event
  "iebc_producer_key_id": "spiffe-svid:fp",
  "iebc_signature": "ed25519:base64...",   // over canonical JSON minus this field
  "iebc_election_id": "2027-general",
  "iebc_data_class": "B"
}
```

`time` is legally authoritative. `iebc_hlc` is for causal ordering across services.

---

## Type naming

`ke.iebc.<bounded_context>.<event_name>.v<major>`

- Lowercase, snake_case for event name.
- Version is monotonic integer, never reused.
- Past tense verbs: `form34a_captured`, `kit_dispatched`, `credential_issued`, `tally_published`, `result_declared`, `seal_broken`.

Examples:
- `ke.iebc.rts.form34a_captured.v1`
- `ke.iebc.rts.form34a_amended.v1`
- `ke.iebc.rts.tally_published.v1`
- `ke.iebc.custody.kit_dispatched.v1`
- `ke.iebc.credential.observer_credential_issued.v1`
- `ke.iebc.declaration.result_declared.v1`
- `ke.iebc.transparency.tree_head_signed.v1`

---

## Topic naming

`iebc.<context>.<aggregate>.v<major>`

- Topic per aggregate + major version. All types for that aggregate's lifecycle share the topic.
- Consumers filter by `type` field.
- Partition count by volume tier: 12 (standard) / 32 (medium) / 64 (high). Never reduce.
- RF=3, min.insync.replicas=2, unclean.leader.election.enable=false, acks=all.
- Partition key = `iebc_stream_id`.

---

## Retention by topic class

| class | example | hot | cold |
|---|---|---|---|
| Event-source (consensus) | `iebc.rts.*`, `iebc.custody.*`, `iebc.credential.*`, `iebc.declaration.*`, `iebc.ballot_serial.*` | infinite | MinIO via Kafka Tiered Storage after 30d |
| Operational | `iebc.turnout.*`, `iebc.grievance.*`, `iebc.dispatch.*` | 30d | none |
| Analytics | `iebc.kit.heartbeat.v1`, `iebc.queue.observation.v1` | 7d | none |
| DLQ / tamper | `<topic>.dlq`, `<topic>.tamper` | 90d | none |

Compaction reserved for state topics (`iebc.config.*.v1`, `iebc.feature_flag.v1`).

---

## Delivery semantics by consumer purpose

| purpose | semantic |
|---|---|
| Event-sourced projection | at-least-once + idempotent by event.id |
| Transparency log ingest | at-least-once + dedupe by (source, id) |
| Real-time LiveView fan-out | at-most-once acceptable |
| Audit sink | at-least-once + idempotent |
| **Tally aggregation** | **exactly-once (Kafka transactions)** |
| **Result declaration** | **exactly-once** |
| **Credential issuance** | **exactly-once** |

Use exactly-once only for the three cases listed. Everywhere else, idempotent-at-least-once is the default.

---

## Signing rules

- Every event on **event-source and credential** topics MUST carry `iebc_signature`.
- Signature is Ed25519 over canonical JSON (RFC 8785) of the envelope minus `iebc_signature`.
- Producer key is SPIFFE-issued, rotated hourly, HSM-anchored trust chain.
- Consumers of signed topics MUST verify before applying. Verification failures ship to `<topic>.tamper` and page immediately.
- Non-signed topics (analytics, operational fan-out) are advisory, not consensus. Do not use them for legally binding decisions.

---

## Schema evolution

- Backward-compatible additions only within a major version: add optional fields with defaults. Never remove or rename.
- Breaking change = new major version (`v2`). Producer emits `v2` alongside `v1` until every consumer has migrated.
- Never rewrite historical events. Errors are corrected by new compensating events.
- CI runs backward-compatibility diff on every schema PR.

---

## Correlation & causation

- **Correlation ID**: trace root. If your service originates a workflow, mint a fresh UUIDv7. If your service is reacting to an incoming event, propagate the incoming `iebc_correlation_id`.
- **Causation ID**: the `event.id` of the event that directly caused this one. If originating (no upstream event), set to null. Enables reconstructing the causal DAG.

The `iebc-events-{rs,py,ex}` library propagates both automatically when you emit from within a consumer handler.

---

## What does NOT go on Kafka

- Bulk data transfer — use MinIO signed URLs.
- Request-response — use gRPC or REST.
- Long-running orchestration — use in-service state machines (Oban / arq / tokio task).
- Session state — use Valkey.
- Hourly-changing config — use Kubernetes ConfigMap or a feature-flag service.
- Public read fan-out — use signed static tarballs via CDN edge (ADR-0011 Tier 1).

Kafka is for events. If it's not an event, do not put it there.

---

## Related

- **ADR-0015** — canonical decision text and full contract.
- **ADR-0013** — event-sourcing boundaries (which modules emit event-source topics).
- **ADR-0016** — PII data classes referenced by `iebc_data_class`.
- **ADR-0017** — transparency log ingests event-source topics.
- `docs/runbooks/consumer-lag-response.md`
- `docs/runbooks/tamper-topic-response.md`
- `docs/runbooks/replay-projection-rebuild.md`
- `docs/runbooks/schema-migration.md`
