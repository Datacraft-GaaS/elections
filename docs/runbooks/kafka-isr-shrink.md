# Kafka ISR shrink response

## Purpose

Restore In-Sync Replicas (ISR) to quorum when one or more brokers fall behind, preventing further data loss and unblocking producers that are waiting on `acks=all`.

## Prerequisites

- `kafka-topics.sh`, `kafka-consumer-groups.sh`, and `kafka-log-dirs.sh` available on the ops jump host.
- Broker hostnames or bootstrap address stored as `$KAFKA_BOOTSTRAP` (e.g., `kafka-0.eel-system:9092,kafka-1.eel-system:9092`).
- Access to the Kubernetes cluster to restart pods if needed.
- An incident ticket open as `<TICKET>`.

## Commands

### Step 1: detect under-replicated partitions

```bash
kafka-topics.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --under-replicated-partitions
# lists every partition where ISR count < replication factor
```

### Step 2: identify the lagging broker

```bash
kafka-topics.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --under-min-isr-partitions
# note the broker IDs in the "Replicas" column that are absent from "Isr"
```

### Step 3: check broker health

```bash
kubectl get pods -n eel-system -l app.kubernetes.io/name=kafka
# if a pod is in CrashLoopBackOff or Pending, address that first
kubectl logs -n eel-system <BROKER_POD> --tail=50
```

### Step 4: restart the lagging broker (if stuck)

```bash
kubectl rollout restart statefulset/<KAFKA_STATEFULSET> -n eel-system
# or target a specific pod:
kubectl delete pod <BROKER_POD> -n eel-system
# Kubernetes will reschedule; Kafka will begin replication catch-up
```

### Step 5: monitor ISR recovery

```bash
watch -n 5 'kafka-topics.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --under-replicated-partitions | wc -l'
# wait for output to reach 0
```

### Step 6: check consumer lag for outbox topics

```bash
kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --group eel-outbox-relay
# if lag > 0, proceed to step 7; otherwise skip
```

### Step 7: replay outbox if consumer lag detected

```bash
# identify the last committed offset before the ISR event:
kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --group eel-outbox-relay
# reset to the safe offset (adjust <TOPIC> and <OFFSET>):
kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --group eel-outbox-relay --topic <TOPIC> \
  --reset-offsets --to-offset <OFFSET> --execute
# restart the outbox relay:
kubectl rollout restart deployment/eel-outbox-relay -n eel-system
```

### Step 8: verify no data loss via offset comparison

```bash
# compare end offsets before and after:
kafka-log-dirs.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --topic-list <TOPIC> --describe \
  | grep '"size"' | awk -F: '{sum+=$2} END {print sum}'
# must match the expected record count from application metrics
```

### Step 9: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.kafka-isr-shrink.resolved.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. `kafka-topics.sh --under-replicated-partitions` returns no output.
2. All consumer groups show zero lag.
3. No dead-letter events on `eel.dlq.*` topics since the incident.
4. Application metrics (bvr-svc, rts-svc event counts) show no gap.

## Rollback procedure

ISR recovery is self-healing once the broker is back. If the broker disk is corrupted, delete the broker's PersistentVolumeClaim and allow Kafka to re-replicate from scratch. This takes time proportional to topic size; notify dependents of extended unavailability.

If data loss is confirmed (offsets missing from all replicas), escalate to Database Lead immediately. Do not attempt to replay from outbox without confirming the extent of loss.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- Kafka/Streaming Lead (on-call via PagerDuty rotation)
- DR Coordinator (contact via war room bridge) if data loss is confirmed

## Estimated duration

15 to 45 minutes depending on replication backlog size.

## Chaos-drill reference

This runbook is exercised by: `chaos/scenarios/eel-kafka-isr-shrink.yaml`

Related: `docs/runbooks/consumer-lag-response.md`, `docs/runbooks/event-replay.md`
