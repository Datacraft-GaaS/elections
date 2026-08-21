# DR failback: mirror site to primary site

## Purpose

Controlled return of the primary site to active status after it has recovered from a failure that triggered `dr-failover-primary-to-mirror.md`. This runbook assumes mirror is currently the active site.

## Prerequisites

- Primary site infrastructure is fully recovered and tested: VMs/nodes healthy, Patroni cluster responding, Kafka brokers online.
- MirrorMaker2 is replicating from mirror back to primary (configured in step 6 of the failover runbook).
- Primary site Patroni cluster has caught up to mirror (replication lag below 30 seconds).
- An incident ticket is open or the original failover ticket is used for continuity. Record the ticket ID as `<TICKET>`.
- Two authorised officers present.
- `tools/dr-failover` binary available on the ops jump host.

## Commands

### Step 1: validate primary site health

```bash
dr-failover preflight --site primary --ticket <TICKET>
# confirms Patroni, Kafka brokers, and core services are healthy on primary
patronictl -c /etc/patroni/primary.yaml list
# confirm replica lag is < 30s
```

### Step 2: freeze writes on mirror

```bash
dr-failover freeze-primary --site mirror --ticket <TICKET>
# sets write-gate to "frozen" on mirror; services enter read-only mode
```

### Step 3: wait for replication catch-up

```bash
watch -n 2 'patronictl -c /etc/patroni/primary.yaml list'
# wait until primary replica lag reaches 0
kafka-consumer-groups.sh --bootstrap-server primary-kafka:9092 \
  --describe --group eel-mm2-mirror-to-primary
# confirm offsets are fully replicated
```

### Step 4: promote primary Patroni

```bash
patronictl -c /etc/patroni/primary.yaml failover eel-pg --master auto --force
patronictl -c /etc/patroni/primary.yaml list
# confirm primary node shows "Leader"
```

### Step 5: restore DNS and anycast to primary

```bash
dr-failover update-dns --target primary --ticket <TICKET>
dig +short api.eel.io
# must resolve to primary site IP
```

### Step 6: restore MirrorMaker2 direction (mirror as standby)

```bash
dr-failover flip-mm2 --source primary --destination mirror --ticket <TICKET>
# resets MirrorMaker2 to normal direction: primary to mirror
```

### Step 7: unfreeze writes on primary

```bash
dr-failover unfreeze --site primary --ticket <TICKET>
```

### Step 8: decommission mirror as active

```bash
dr-failover deactivate-site --site mirror --ticket <TICKET>
# mirror returns to standby; read traffic may remain on mirror via local caches
```

### Step 9: run reconciliation

```bash
dr-failover reconcile --ticket <TICKET>
# compares event counts on primary and mirror tlog; reports any delta
```

### Step 10: emit tlog failback entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.dr-failback.mirror-to-primary.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"officer_a\":\"<ID_A>\",\"officer_b\":\"<ID_B>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. `dr-failover status --site primary` returns `ACTIVE`.
2. `dig +short api.eel.io` resolves to a primary site IP.
3. Reconciliation report (step 9) shows zero delta.
4. Smoke test: `dr-failover smoke-test --site primary --ticket <TICKET>` returns `ALL_PASS`.
5. Transparency log contains the failback leaf.

## Rollback procedure

If failback fails at any step before step 7 (unfreeze), re-freeze the primary, restore mirror DNS, and unfreeze mirror to resume mirror-active operation. Then investigate the root cause before re-attempting failback.

If failback fails after step 7, open a P0 incident per `incident-response-p0.md`. Both sites may briefly be in an inconsistent state; use `dr-failover reconcile` to assess data delta.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- DR Coordinator (contact via war room bridge)
- Database Lead for Patroni issues (on-call via PagerDuty rotation)

## Estimated duration

20 to 40 minutes depending on replication catch-up time.

## Chaos-drill reference

This runbook is exercised as the recovery phase of: `chaos/scenarios/eel-site-partition-primary-mirror.yaml`
