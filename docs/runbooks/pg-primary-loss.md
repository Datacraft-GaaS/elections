# PostgreSQL primary loss: Patroni-driven promotion

## Purpose

Respond to an unplanned PostgreSQL primary node failure. Patroni will auto-promote a replica within its TTL (default 30 seconds). This runbook covers detection, verification of promotion, post-failover health checks, and connection string updates.

RTO target: 3 minutes from detection to write availability.

## Prerequisites

- `patronictl` installed on the ops jump host, configured with `/etc/patroni/primary.yaml`.
- `pgbackrest` installed and configured at `/etc/pgbackrest/pgbackrest.conf`.
- Access to the Kubernetes cluster where services are deployed.
- An incident ticket is open. Record as `<TICKET>`.

## Commands

### Step 1: detect the failure

```bash
patronictl -c /etc/patroni/primary.yaml list
# look for: primary node absent or showing "stopped"
# Patroni will begin leader election automatically; TTL is 30s
```

### Step 2: monitor auto-promotion

```bash
watch -n 2 'patronictl -c /etc/patroni/primary.yaml list'
# wait for a replica to transition to "Leader" state
# if no promotion after 60s, proceed to manual promotion below
```

### Step 3: manual promotion (if auto-promotion stalled)

```bash
# identify the replica with lowest lag:
patronictl -c /etc/patroni/primary.yaml list
# promote it explicitly:
patronictl -c /etc/patroni/primary.yaml failover eel-pg \
  --master <REPLICA_NAME> --force
```

### Step 4: verify pgBackRest post-failover

```bash
pgbackrest --stanza=eel info
# confirm last backup is recent (within RPO window)
pgbackrest --stanza=eel check
# must complete without errors
```

### Step 5: check replication lag on remaining replicas

```bash
patronictl -c /etc/patroni/primary.yaml list
# lag column should be 0 or near-0 for all replicas after a few minutes
```

### Step 6: verify connection strings

```bash
# Services use the Patroni-managed service endpoint; confirm it points to new leader:
kubectl get endpoints eel-pg-primary -n eel-system
# endpoint IP must match the newly promoted node
```

### Step 7: run service health checks

```bash
kubectl get pods -n eel-system -l app.kubernetes.io/component=database-client
# all pods should be Running; any CrashLoopBackOff indicates connection string issues
for svc in bvr-svc bvi-svc rts-svc abis-svc; do
  kubectl exec -n eel-system deploy/$svc -- \
    wget -qO- http://localhost:8080/health | grep '"db":"ok"'
done
```

### Step 8: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.pg-primary-loss.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"new_leader\":\"<NODE>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. `patronictl list` shows exactly one node as `Leader`.
2. All service health endpoints return `"db":"ok"`.
3. `pgbackrest check` exits 0.
4. No Kafka dead-letter events accumulating (check `eel.dlq.*` topics).
5. Transparency log contains the pg-primary-loss leaf.

## Rollback procedure

There is no rollback for Patroni promotion. The failed primary node becomes a replica once it recovers. To re-elect it as primary if required, use `patronictl switchover` after it has caught up (lag = 0). Confirm with the Database Lead before doing so.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- Database Lead (on-call via PagerDuty rotation)
- DR Coordinator (contact via war room bridge) if replication lag exceeds RPO

## Estimated duration

2 to 5 minutes for auto-promotion. Up to 15 minutes if manual intervention is required.

## Chaos-drill reference

This runbook is exercised by: `chaos/scenarios/eel-pg-primary-kill-patroni-failover.yaml`

Also referenced in: `docs/runbooks/pgbackrest-pitr.md`
