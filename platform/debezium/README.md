# platform/debezium/

Debezium CDC (Change Data Capture) connector configuration for the eel IEMS. Implements the outbox pattern: services write to a local `eel_outbox` table, Debezium captures changes and publishes them to Kafka.

## Contents

| File | Description |
|---|---|
| `connector-template.json` | Template for a Debezium PostgreSQL connector (filled per service) |
| `local/` | Kustomize overlay for kind (single connector, reduced poll interval) |

## Connector per service

Each T1 (Tier 1) PostgreSQL cluster has one Debezium connector. Connector configs are generated from `connector-template.json` by the service deploy process and applied to the Kafka Connect REST API.

Example connector name: `eel-rts-svc-outbox`

```bash
# Apply a connector:
curl -X POST http://kafka-connect.eel-kafka:8083/connectors \
  -H "Content-Type: application/json" \
  -d @services/rts-svc/deploy/debezium-connector.json

# List connectors:
curl http://kafka-connect.eel-kafka:8083/connectors

# Check connector status:
curl http://kafka-connect.eel-kafka:8083/connectors/eel-rts-svc-outbox/status
```

## Outbox table contract

Each service that produces events must have:

```sql
CREATE TABLE eel_outbox (
    id          UUID PRIMARY KEY,
    aggregate   TEXT NOT NULL,
    event_type  TEXT NOT NULL,
    payload     JSONB NOT NULL,
    created_at  TIMESTAMPTZ DEFAULT NOW()
);
```

Debezium reads this table via PostgreSQL logical replication (publication `eel_outbox_pub`).

## Transform pipeline

Debezium applies the Outbox Event Router SMT to route events to the correct Kafka topic based on `aggregate` and `event_type` columns.

## Overlays

| Overlay | Description |
|---|---|
| `local/` | kind: single connector, 500ms poll interval, no SSL |

## Operator responsibilities

- Logical replication slots accumulate WAL if a connector falls behind. Monitor `pg_replication_slots` lag in the Grafana dashboard.
- If a connector fails, WAL accumulation can fill the PostgreSQL data volume. Set `wal_keep_size = 1GB` as the safety limit.
- Connector offsets are stored in Kafka topic `__debezium_offsets`. Do not delete this topic.

## Related docs

- `platform/kafka/README.md`: Kafka cluster receiving the events
- `platform/postgres/README.md`: source PostgreSQL clusters
- `docs/EVENT-CONVENTIONS.md`: outbox contract and event naming
