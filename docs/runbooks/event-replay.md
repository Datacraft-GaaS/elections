# Event replay: manual replay from outbox or Kafka

## Purpose

Replay events from the outbox or a Kafka topic when a consumer missed them or a projection needs to be rebuilt. Replay is a read-only operation on the source; it re-emits events to a consumer group. It must be idempotent: consumers must tolerate seeing the same event twice (EC-DAT-01).

## Prerequisites

- `kafka-consumer-groups.sh` and `kafka-topics.sh` available on the ops jump host.
- Broker bootstrap address stored as `$KAFKA_BOOTSTRAP`.
- The consumer group to replay is identified as `<GROUP>` and the topic as `<TOPIC>`.
- An incident ticket open as `<TICKET>`.
- Database Lead approval before any offset reset.

## Commands

### Step 1: confirm the replay target

```bash
kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --group <GROUP>
# record the current committed offsets as the rollback point
```

### Step 2: determine the safe offset

```bash
# find the earliest offset that must be replayed (e.g. the last good event):
kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --group <GROUP>
# or query the topic for the desired start offset:
kafka-run-class.sh kafka.tools.GetOffsetShell \
  --bootstrap-server $KAFKA_BOOTSTRAP --topic <TOPIC> --time -1
```

### Step 3: reset the consumer group to the safe offset

```bash
kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --group <GROUP> --topic <TOPIC> \
  --reset-offsets --to-offset <OFFSET> --execute
# confirm the reset:
kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --group <GROUP>
```

### Step 4: restart the consumer to trigger replay

```bash
kubectl rollout restart deployment/<CONSUMER> -n eel-system
# the consumer re-reads from the reset offset and re-processes events
```

### Step 5: monitor replay progress

```bash
watch -n 5 'kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --group <GROUP> | awk "{sum+=\$6} END {print sum}"'
# lag should climb then fall to 0 as the consumer catches up
```

### Step 6: verify idempotent processing

```bash
# confirm no duplicate side effects: check the consumer's dedupe store
# (e.g. bvr-svc idempotency table) for the replayed event keys
# confirm no dead-letter events accumulated:
kafka-topics.sh --bootstrap-server $KAFKA_BOOTSTRAP --list | grep '^eel.dlq'
```

### Step 7: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.event-replay.completed.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"group\":\"<GROUP>\",\"topic\":\"<TOPIC>\",\"from_offset\":\"<OFFSET>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. Consumer group lag returns to 0 after replay.
2. No duplicate side effects (idempotency store shows no re-execution).
3. No dead-letter events accumulated during replay.
4. Downstream projections and tally are consistent with the replayed events.
5. Transparency log contains the replay-completed leaf.

## Rollback procedure

To roll back a replay, reset the group back to the recorded pre-replay offsets and restart the consumer. Because consumers are idempotent, re-processing is safe; the risk is a consumer that is not idempotent, which would duplicate side effects. If duplicates are detected, stop the consumer and reconcile with the Database Lead before continuing.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- Kafka/Streaming Lead (on-call via PagerDuty rotation)
- Database Lead (on-call via PagerDuty rotation) if offset reset affects a projection

## Estimated duration

15 to 60 minutes depending on backlog size.

## Chaos-drill reference

This runbook is exercised by: `chaos/scenarios/eel-debezium-pause.yaml` (Debezium pause causes outbox lag that must be replayed).

Related: `docs/runbooks/consumer-lag-response.md`, `docs/runbooks/replay-projection-rebuild.md`
