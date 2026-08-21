# platform/kafka/

Kafka KRaft cluster and topic definitions for the eel IEMS event bus. All inter-service communication uses events via this cluster per `docs/EVENT-CONVENTIONS.md`.

## Contents

| Path | Description |
|---|---|
| `topics/` | One YAML file per Kafka topic (topic name, partition count, retention, replication factor) |
| `local/` | Kustomize overlay for kind (single-broker, reduced partitions) |

## Topic naming convention

```
eel.<context>.<aggregate>.<event>.v<n>
```

Examples:
- `eel.rts.results.submitted.v1`
- `eel.bvr.tallies.certified.v1`
- `eel.abis.match.completed.v1`

All topic definitions must have a corresponding event schema in `packages/event-schemas/`.

## Topics (sample)

| Topic | Partitions | Retention | Producers | Consumers |
|---|---|---|---|---|
| `eel.bvi.identification.v1` | 46 | 7 days | bvi-svc | rts-svc, biometric-edge-svc |
| `eel.rts.results.submitted.v1` | 20 | 90 days | rts-svc | transparency-svc, portal-svc |
| `eel.abis.match.completed.v1` | 10 | 30 days | abis-svc | bvr-svc |
| `eel.bvr.tallies.certified.v1` | 10 | 90 days | bvr-svc | transparency-svc, ecc-svc |

Full list: `platform/kafka/topics/`.

## Install

Kafka runs as a KRaft (no ZooKeeper) cluster managed by Strimzi operator:

```bash
helm upgrade --install strimzi strimzi/strimzi-kafka-operator \
  --namespace eel-kafka --create-namespace

kubectl apply -f platform/kafka/
```

## Apply a new topic

```bash
kubectl apply -f platform/kafka/topics/eel.new.event.v1.yaml
```

Topic files are reconciled by the Strimzi Topic Operator automatically.

## Verify

```bash
# List all topics:
kubectl exec -n eel-kafka kafka-0 -- \
  bin/kafka-topics.sh --bootstrap-server localhost:9092 --list

# Check consumer lag:
kubectl exec -n eel-kafka kafka-0 -- \
  bin/kafka-consumer-groups.sh --bootstrap-server localhost:9092 --describe --all-groups
```

## Overlays

| Overlay | Description |
|---|---|
| `local/` | kind: single broker, 1 partition per topic, no tiered storage |

## Operator responsibilities

- Partition count is immutable after creation. Plan for peak load (46,229 kits concurrent) at topic definition time.
- Tiered storage to Ceph RGW is configured for topics with retention > 30 days.
- Schema compatibility is enforced by Karapace (configured in `platform/karapace/`).

## Related docs

- `platform/karapace/README.md`: schema registry
- `platform/debezium/README.md`: CDC connectors feeding this cluster
- `platform/mirrormaker2/README.md`: DR replication to mirror cluster
- `docs/EVENT-CONVENTIONS.md`: topic naming and schema versioning rules
