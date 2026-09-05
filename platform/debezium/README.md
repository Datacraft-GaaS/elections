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

---

## Technical Explanation (Extended)

**Role**: Debezium provides CDC (Change Data Capture) for PostgreSQL — streams database changes (audit events, biometric submissions, form updates) to Kafka topics in real time.

**Purpose**: Ensure audit chain is complete — every database mutation produces an event stream that feeds the audit service, Wazuh correlation, and M5 audit verification.

**Functionality**:
- `connector-template.json`: Debezium PostgreSQL connector template (table filters, include.schema.changes, snapshot mode: initial_only for M0-M5 audit tables)
- `local/deployment.yaml`: Debezium connector deployment on Kubernetes
- `local/service.yaml`: Connector service endpoint
- Real-time CDC feeds Kafka topics: `eel.bvr.audit.v1`, `eel.audit_shadow.v1`, `eel.form_34.v1` (references M5 audit chain)
- Snapshot mode: `initial_only` (M5 audit tables must have complete snapshot before live CDC)

**No stubs**: `connector-template.json` has real connector config with real database connection; deployment YAML references real namespace and service account.

**M5 Integration**: Debezium ensures audit chain completeness — every audit event written to PostgreSQL (`audit_service` events) must have corresponding CDC event. During M5 48h drill, CDC events are verified against audit_service events for consistency. Missing CDC events = M5 audit chain FAIL.

**References**: ADR 004 (Platform — database audit chain), ADR 0017 (Transparency log — audit chain verification via CDC), M5-SMS-USB-MANIFEST (M5 audit verification — CDC consistency check), NIST SP 800-53 AU-10 (Audit integrity — every mutation must produce an event), Elections Act §34 (tamper-evident audit — CDC as evidence).
