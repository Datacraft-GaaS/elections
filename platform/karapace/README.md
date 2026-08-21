# platform/karapace/

Karapace schema registry for the eel IEMS Kafka cluster. Enforces event schema compatibility and serves schemas to producers and consumers.

Karapace is the Aiven fork of the Confluent Schema Registry with an identical API surface, chosen for its Apache 2.0 license.

## Contents

| Path | Description |
|---|---|
| `local/` | Kustomize overlay for kind (standalone mode, in-memory topic) |

## Schema subjects

Each Kafka topic has a corresponding schema subject following the `TopicNameStrategy`:

```
eel.rts.results.submitted.v1-value
eel.bvr.tallies.certified.v1-value
```

Schemas are in `packages/event-schemas/<ctx>/<agg>/<event>/v<n>.schema.json` (JSON Schema Draft 7).

## Compatibility mode

All subjects use `FORWARD_TRANSITIVE` compatibility:
- Producers can add new optional fields.
- Consumers must tolerate unknown fields.
- Removing required fields or changing types is rejected by the registry.

## Register a schema

```bash
# Register via Karapace REST API:
curl -X POST http://karapace.eel-kafka:8081/subjects/eel.rts.results.submitted.v1-value/versions \
  -H "Content-Type: application/json" \
  -d '{"schema": "<json-schema-string>"}'

# Check compatibility before registering:
curl -X POST http://karapace.eel-kafka:8081/compatibility/subjects/eel.rts.results.submitted.v1-value/versions/latest \
  -H "Content-Type: application/json" \
  -d '{"schema": "<new-schema-string>"}'
```

## Overlays

| Overlay | Description |
|---|---|
| `local/` | kind: single instance, topics backed by local Kafka |

## Operator responsibilities

- Schema registration is idempotent for identical schemas. Re-registering the same schema version returns the existing ID.
- The `_schemas` Kafka topic holds all schema data; it must not be deleted or compacted.
- Mirror the schema registry to the DR cluster via MirrorMaker2 topic replication of `_schemas`.

## Related docs

- `platform/kafka/README.md`: Kafka cluster
- `platform/debezium/README.md`: CDC connectors that produce schematized events
- `packages/event-schemas/README.md`: event schema source files
- `docs/EVENT-CONVENTIONS.md`: schema versioning rules
