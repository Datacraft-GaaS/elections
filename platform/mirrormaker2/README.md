# platform/mirrormaker2/

Kafka MirrorMaker2 configuration for DR replication of Kafka topics from the primary cluster to the mirror cluster.

## Contents

| File | Description |
|---|---|
| `mirrormaker2.yaml` | Strimzi `KafkaMirrorMaker2` resource (active-passive, primary to mirror) |

## Replication model

- Direction: primary (`eel-kafka.primary`) to mirror (`eel-kafka.mirror`).
- Topics replicated: all `eel.*` topics. The `_schemas` topic (Karapace) is also replicated to keep schema registries in sync.
- Consumer group offsets are replicated so failover consumer reconnection is lag-minimal.
- Replication factor on the mirror: 3 (same as primary).

## Apply

```bash
kubectl apply -f platform/mirrormaker2/mirrormaker2.yaml -n eel-kafka
```

## Verify replication lag

```bash
# Check MirrorMaker2 connector status:
kubectl exec -n eel-kafka kafka-0 -- \
  bin/kafka-consumer-groups.sh \
  --bootstrap-server localhost:9092 \
  --describe --group mirrormaker2-eel-kafka

# Replication lag metric in Grafana: kafka_mirrormaker2_record_lag
```

## Failover procedure

During DR failover (`eel-dr-failover execute`), MirrorMaker2 is:
1. Stopped (write quiesce phase).
2. Consumer group offsets translated using `RemotePartitionSyncTask`.
3. Primary Kafka bootstraps re-pointed to mirror via `KAFKA_BOOTSTRAP_SERVERS` update.

See `docs/runbooks/dr-failover.md` for the full sequence.

## Operator responsibilities

- MirrorMaker2 lag alert fires at > 10,000 messages. Investigate Strimzi operator logs and network connectivity between sites.
- Do not replicate topics that contain transient state (e.g., Valkey keyspace notifications). Check `mirrormaker2.yaml` topic exclusions.
- The `_schemas` topic replication keeps Karapace in sync. Verify schema IDs match on both sides after any schema registration.

## Related docs

- `platform/kafka/README.md`: primary cluster
- `platform/karapace/README.md`: schema registry sync
- `tools/dr-failover/README.md`: automated failover tool
- `docs/runbooks/dr-failover.md`: full DR failover runbook
