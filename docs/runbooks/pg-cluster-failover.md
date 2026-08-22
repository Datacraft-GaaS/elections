# PostgreSQL cluster failover: planned Patroni switchover

## Purpose

Perform a planned PostgreSQL primary switchover, for example before maintenance on the current primary node. A planned switchover is controlled and low-risk, unlike an unplanned primary loss (see `pg-primary-loss.md`). The goal is to promote a replica to primary with minimal downtime and no data loss.

## Prerequisites

- `patronictl` installed on the ops jump host, configured with `/etc/patroni/primary.yaml`.
- All replicas are healthy and caught up (lag = 0).
- An incident ticket open as `<TICKET>` (maintenance tickets are pre-approved).
- Database Lead present to witness the switchover.

## Commands

### Step 1: confirm cluster health

```bash
patronictl -c /etc/patroni/primary.yaml list
# confirm exactly one Leader and all replicas at lag 0
```

### Step 2: choose the target replica

```bash
# select the replica with the lowest lag and most recent data:
patronictl -c /etc/patroni/primary.yaml list
# note the target replica name as <TARGET>
```

### Step 3: perform the switchover

```bash
patronictl -c /etc/patroni/primary.yaml switchover eel-pg \
  --master <CURRENT_PRIMARY> --candidate <TARGET> --force
# Patroni promotes <TARGET> and demotes the current primary to replica
```

### Step 4: verify the new primary

```bash
patronictl -c /etc/patroni/primary.yaml list
# confirm <TARGET> is now Leader
kubectl get endpoints eel-pg-primary -n eel-system
# confirm the endpoint points to the new primary node
```

### Step 5: verify replication lag

```bash
patronictl -c /etc/patroni/primary.yaml list
# all replicas should return to lag 0 within a few minutes
```

### Step 6: run service health checks

```bash
for svc in bvr-svc bvi-svc rts-svc abis-svc; do
  kubectl exec -n eel-system deploy/$svc -- \
    wget -qO- http://localhost:8080/health | grep '"db":"ok"'
done
```

### Step 7: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.pg-cluster-failover.completed.v1\",\"payload\":{\"ticket\":\"<TICKET>\",\"new_primary\":\"<TARGET>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. `patronictl list` shows exactly one Leader, which is the target.
2. All service health endpoints return `"db":"ok"`.
3. All replicas return to lag 0.
4. No data loss (compare sequence numbers before and after).
5. Transparency log contains the switchover-completed leaf.

## Rollback procedure

To switch back, repeat the switchover with the original primary as the candidate, after it has caught up (lag = 0). Confirm with the Database Lead before doing so. A planned switchover is reversible; an unplanned failover is not.

## Escalation contacts

- Database Lead (on-call via PagerDuty rotation)
- Site Reliability Lead (on-call via PagerDuty rotation)
- DR Coordinator (contact via war room bridge) if the switchover fails mid-way

## Estimated duration

5 to 10 minutes.

## Chaos-drill reference

This is a planned procedure, not a chaos scenario. It is related to `docs/runbooks/pg-primary-loss.md` (unplanned failover) and `docs/runbooks/pgbackrest-pitr.md` (restore).
