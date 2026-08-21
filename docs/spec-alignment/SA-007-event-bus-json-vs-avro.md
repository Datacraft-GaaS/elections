# SA-007: Event bus — JSON + zstd + per-aggregate topics vs Avro + domain topics

Date: 2026-08-21
Applies to: Datacraft bid documents under work_docs/01-Projects/IEBC/sw/ (our own earlier draft)
Status: decided (ADR takes precedence; sw/ docs to be edited)

## Earlier draft (sw/) says

`sw/ARCHITECTURE.md` §9 pins:

- **Avro** as the Kafka payload encoding, with **Confluent Schema Registry**.
- ~10 topics organised by domain.
- 7-day hot retention + compacted-topic export to MinIO for permanent archive.

## Decision (per ADRs)

- **JSON + zstd on the wire**, one encoding, canonical JSON (RFC 8785) for signing.
- **~500 topics** organised per aggregate + major version (`iebc.<context>.<aggregate>.v<major>`).
- **Karapace** (Aiven Apache 2.0 fork of Confluent SR — supports both Avro and JSON Schema).
- **90-day hot retention** (RFP-aligned per PRE_BID D13) with Kafka Tiered Storage to MinIO thereafter.

## RFP status

**Not RFP-binding on payload encoding or topic granularity.** tender.txt names neither Avro, JSON, MessagePack, nor specific topic taxonomy. RFP DOES mandate 90-day online log retention (tender.txt Sec V + PRE_BID D13) — the 7-day figure in ARCHITECTURE.md §9 is Datacraft-drafted and short of the RFP requirement.

## Rationale

### JSON + zstd

1. **Debuggability**. Election-critical events are legally-relevant artefacts. When an investigator (auditor, court, IAB) reads an event, JSON is human-legible; Avro requires the schema plus tooling to decode. Debug-ability is not marketing — it is auditor-defensibility.
2. **Signing model.** Canonical JSON (RFC 8785 JCS) is a well-specified deterministic canonicalisation. Cross-language Ed25519 verification works out of the box. Canonical Avro (via cross-encoding) is possible but adds a codec dependency to every verifier.
3. **One codec path**. Two encodings (Avro + something-else for signing) doubles the surface. One encoding is simpler across ~100 services and ~500 topics.
4. **Client footprint**. Mobile clients (Flutter M13/M29-M34) get JSON natively; adding Avro would import an Avro codec into every mobile app.
5. **Karapace parity**. If we adopt JSON Schema in Karapace, we get schema validation + evolution + registry integration matching what Avro-in-Confluent-SR offers — without the vendor licence issue (see SA-001).

### Per-aggregate topics (~500) vs domain topics (~10)

1. **Consumer isolation**. Ten domain topics means every consumer of any results event subscribes to the same "results" topic and filters. High-volume filtering at the consumer side burns CPU and network. Per-aggregate topics let each consumer subscribe to only what it cares about.
2. **Blast-radius bounded**. A misbehaving producer flooding one aggregate's topic does not backpressure other aggregates' consumers.
3. **Retention granularity**. Different aggregates have different retention needs (event-source infinite, operational 30d, analytics 7d). Per-aggregate topics let each be tuned; per-domain forces a lowest-common-denominator retention.
4. **Kafka Kraft handles thousands of topics trivially**; ~500 is not close to any operational ceiling.

### Trade-off honestly stated

Avro is genuinely tighter (smaller payload, schema-enforced at write time) and matches Kafka ecosystem expectations well. If Datacraft has already invested in Avro schemas for M01-M28 (per `sw/API-CATALOG.md`), the migration cost of moving to JSON is real.

## Edits to apply to sw/

- **On encoding**: if Datacraft has existing Avro artefacts for M01-M28 already generated, we accept **Avro via Karapace** (Apache 2.0 fork; solves the licence-hygiene concern from SA-001) as the wire format for existing topics, with new topics allowed either encoding. Signing scheme becomes canonical-Avro (deterministic Avro binary encoding).
- **On topic granularity**: we compromise on **medium-granularity — one topic per (bounded-context, event-class)** — roughly 50-100 topics. Less than 500, more than 10. Preserves consumer isolation for the biggest event classes without exploding topic count.
- **On retention**: **90 days hot is non-negotiable** (RFP-binding, PRE_BID D13). Datacraft's 7-day figure must be revised regardless of the other outcomes.

## Related

ADR-0015 (event bus conventions), ADR-0003 (Karapace), ADR-0023 (RFP-alignment), SA-001 (Karapace vs Confluent SR licence).
