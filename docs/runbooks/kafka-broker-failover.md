# Kafka broker failover

## Purpose

Respond to a Kafka broker node failure. Kafka is designed to survive broker loss as long as the ISR quorum holds; this runbook covers detection, verification that the cluster remains available, and recovery of the failed broker. For under-replicated partitions that threaten quorum, use `kafka-isr-shrink.md` instead.

## Prerequisites

- `kafka-topics.sh`, `kafka-consumer-groups.sh`, and `kafka-log-dirs.sh` available on the ops jump host.
- Broker bootstrap address stored as `$KAFKA_BOOTSTRAP`.
- Access to the Kubernetes cluster to inspect and restart broker pods.
- An incident ticket open as `<TICKET>`.

## Commands

### Step 1: detect the failed broker

```bash
kubectl get pods -n eel-system -l app.kubernetes.io/name=kafka
# identify the broker pod that is not Running
kubectl logs -n eel-system <BROKER_POD> --tail=50
# confirm the failure mode (disk, network, or crash)
```

### Step 2: verify cluster availability

```bash
kafka-topics.sh --bootstrap-server $KAFKA_BOOTSTRAP --list
# if this succeeds, the cluster is available; producers are not blocked
kafka-topics.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --under-replicated-partitions
# if this returns partitions, ISR has shrunk; monitor closely
```

### Step 3: confirm producers are not blocked

```bash
# producers with acks=all block only if ISR drops below min.insync.replicas:
kafka-topics.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --under-min-isr-partitions
# if this returns partitions, producers may be blocked; see kafka-isr-shrink.md
```

### Step 4: recover the failed broker

```bash
# if the broker pod is stuck, delete it and let Kubernetes reschedule:
kubectl delete pod <BROKER_POD> -n eel-system
# if the broker's PersistentVolumeClaim is corrupted, delete it to force re-replication:
kubectl delete pvc <BROKER_PVC> -n eel-system
# the broker re-joins and begins replication catch-up
```

### Step 5: monitor ISR recovery

```bash
watch -n 5 'kafka-topics.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --under-replicated-partitions | wc -l'
# wait for output to reach 0
```

### Step 6: verify consumer lag

```bash
kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --group eel-outbox-relay
# if lag > 0, follow consumer-lag-response.md
```

### Step 7: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.kafka-broker-failover.resolved.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"broker\":\"<BROKER_ID>\",\"root_cause\":\"<CAUSE>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. All brokers are Running and ISR is at quorum (no under-replicated partitions).
2. Producers are not blocked (no under-min-ISR partitions).
3. Consumer groups show zero lag.
4. No dead-letter events accumulated during the failover.
5. Transparency log contains the resolved leaf.

## Rollback procedure

There is no rollback for a broker failover; the cluster self-heals once the broker rejoins. If the broker's disk is corrupted, deleting the PVC forces full re-replication, which takes time proportional to topic size. Notify dependents of extended unavailability during re-replication.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- Kafka/Streaming Lead (on-call via PagerDuty rotation)
- DR Coordinator (contact via war room bridge) if ISR drops below quorum

## Estimated duration

15 to 45 minutes depending on replication backlog size.

## Chaos-drill reference

This runbook is exercised by: `chaos/scenarios/eel-kafka-isr-shrink.yaml` (ISR shrink from broker loss).

Related: `docs/runbooks/kafka-isr-shrink.md`, `docs/runbooks/consumer-lag-response.md`
