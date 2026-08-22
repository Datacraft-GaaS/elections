# Consumer lag response

## Purpose

Respond to a Kafka consumer group whose lag exceeds the alert threshold. Lag means a consumer is falling behind the produce rate, which can delay event-driven processing (outbox relay, projections, tally) and, if unbounded, cause data loss or OOM. The goal is to identify the cause, restore the consumer to current, and confirm no events were dropped.

## Prerequisites

- `kafka-consumer-groups.sh` available on the ops jump host.
- Broker bootstrap address stored as `$KAFKA_BOOTSTRAP`.
- Access to the Kubernetes cluster to inspect and restart consumer pods.
- An incident ticket open as `<TICKET>`.

## Commands

### Step 1: identify the lagging group

```bash
kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP --list
# note the group(s) with lag; the alert names the group and topic
```

### Step 2: measure the lag

```bash
kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --group <GROUP>
# note the LAG column per partition; identify hot partitions
```

### Step 3: inspect the consumer pod

```bash
kubectl get pods -n eel-system -l app.kubernetes.io/name=<CONSUMER>
kubectl logs -n eel-system deploy/<CONSUMER> --tail=50
# look for errors, slow processing, or a stuck poll loop
```

### Step 4: check for a hot partition

```bash
# if one partition has disproportionate lag, the key distribution is skewed:
kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --group <GROUP> | sort -k6 -n
# a single partition far ahead of the rest indicates a hot key
```

### Step 5: scale out or restart the consumer

```bash
# if the consumer is CPU-bound, scale out:
kubectl scale deployment/<CONSUMER> --replicas=<N> -n eel-system
# if the consumer is stuck (no progress), restart it:
kubectl rollout restart deployment/<CONSUMER> -n eel-system
```

### Step 6: monitor lag recovery

```bash
watch -n 5 'kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --group <GROUP> | awk "{sum+=\$6} END {print sum}"'
# wait for total lag to reach 0
```

### Step 7: verify no events dropped

```bash
# compare the group's committed offset with the topic end offset:
kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --group <GROUP>
# if the committed offset is behind the end offset, events are still queued, not lost
# confirm no dead-letter events accumulated:
kafka-topics.sh --bootstrap-server $KAFKA_BOOTSTRAP --list | grep '^eel.dlq'
```

### Step 8: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.consumer-lag.resolved.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"group\":\"<GROUP>\",\"root_cause\":\"<CAUSE>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. Total lag for the group is 0.
2. The consumer is processing at the produce rate (no new lag accumulating).
3. No events were dropped (committed offsets are contiguous; no DLQ growth).
4. Downstream projections and tally are current.
5. Transparency log contains the resolved leaf.

## Rollback procedure

If scaling out or restarting does not clear the lag, the consumer may be blocked on a downstream dependency (database, outbox, or external API). Check the consumer logs for the blocking call and address that dependency first. Do not reset offsets unless data loss is confirmed and the Database Lead approves; an incorrect reset can skip valid events.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- Kafka/Streaming Lead (on-call via PagerDuty rotation)
- Service owner of the consuming service if the lag is in a projection or tally consumer

## Estimated duration

15 to 45 minutes depending on backlog size and root cause.

## Chaos-drill reference

This runbook is exercised by: `chaos/scenarios/eel-kafka-isr-shrink.yaml` (ISR shrink causes lag) and `chaos/scenarios/eel-debezium-pause.yaml` (Debezium pause causes outbox lag).

Related: `docs/runbooks/event-replay.md`, `docs/runbooks/kafka-isr-shrink.md`
