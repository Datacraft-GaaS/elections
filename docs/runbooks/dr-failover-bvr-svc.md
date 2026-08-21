# DR Failover Runbook: bvr-svc (Biometric Voter Register)

SLO: RTO <= 60 min, RPO <= 15 min (ADR-0022, T1 non-rts services)

## Prerequisites

- Two-human quorum on comms channel
- `kubectl` context set to active site
- `eel-dr-failover` binary: `tools/dr-failover/target/release/eel-dr-failover`
- Patroni CLI on a T1 database node

## Step 0: Declare incident

1. Page NOC on-call: `#eel-noc` Slack, PagerDuty "EEL DR Failover"
2. Record start time: `START=$(date -u +%Y-%m-%dT%H:%M:%SZ)`
3. Open Jira DR-DRILL-BVR-{YYYY-MM-DD}

## Step 1: Confirm failure scope

```bash
# Are bvr-svc pods healthy?
kubectl -n eel-prod get pods -l app=bvr-svc

# Is the T1 voter-db reachable?
kubectl -n eel-prod exec deploy/bvr-svc -- \
  psql "$VOTER_DB_URL" -c "SELECT count(*) FROM voters WHERE active;" 2>&1

# Mirror replica lag
kubectl -n eel-prod exec deploy/bvr-svc -- \
  psql "$VOTER_DB_MIRROR_URL" -c \
  "SELECT now() - pg_last_xact_replay_timestamp() AS lag;"
```

Gate: if lag > 15 min, record RPO breach.

## Step 2: Review plan

```bash
eel-dr-failover plan --service bvr-svc
```

## Step 3: Dry-run

```bash
eel-dr-failover execute --service bvr-svc --dry-run
```

## Step 4: Promote voter-db replica on mirror site

```bash
patronictl -c /etc/patroni/patroni.yml list

patronictl -c /etc/patroni/patroni.yml failover \
  --master <voter-db-primary> \
  --candidate <voter-db-mirror-replica> \
  --force

watch -n 2 patronictl -c /etc/patroni/patroni.yml list
```

Verification:
```bash
patronictl list | grep -c " Leader"  # must be 1
psql "$VOTER_DB_MIRROR_URL" -c "SELECT pg_is_in_recovery();"  # must return 'f'
```

## Step 5: Update bvr-svc database endpoint

```bash
kubectl -n eel-prod patch configmap bvr-svc-config \
  --patch '{"data":{"VOTER_DB_HOST":"postgres-t1-voter-mirror.eel-prod.svc.cluster.local"}}'

kubectl -n eel-prod rollout restart deploy/bvr-svc
kubectl -n eel-prod rollout status deploy/bvr-svc --timeout=10m
```

## Step 6: Verify voter register integrity

```bash
# Health check
curl -sf https://bvr.eel.internal/healthz | jq .

# Voter count sanity check (compare to pre-failover baseline)
kubectl -n eel-prod exec deploy/bvr-svc -- \
  psql "$VOTER_DB_MIRROR_URL" -c \
  "SELECT count(*) FROM voters WHERE active AND NOT deregistered;" \
  | tee /tmp/voter-count-post-failover.txt

# Diff against last known good count (stored in Jira ticket or runbook baseline)
# If count differs by > 0.1%, escalate to data integrity team
```

## Step 7: Verify biometric index sync

bvr-svc depends on ABIS for deduplication. Confirm ABIS is reachable on the mirror site:

```bash
curl -sf https://abis.eel.internal/healthz | jq .

# If ABIS is on the failed site, fail it over first (see docs/runbooks/dr-failover-abis-svc.md)
```

## Step 8: Verify Kafka topic replication

Voter registration events flow through `eel.bvr.events`. Confirm mirror-site consumers see current events:

```bash
kafka-consumer-groups.sh \
  --bootstrap-server kafka-mirror.eel-prod.svc.cluster.local:9092 \
  --describe \
  --group bvr-projections
# LAG must be < 1 000 within 5 min of failover
```

## Step 9: Record RTO and RPO

```bash
eel-dr-failover report \
  --drill "bvr-svc-$(date +%Y%m%d)" \
  --check-svc bvr-svc \
  --format json | tee /tmp/dr-report-bvr-$(date +%Y%m%d).json
```

Paste report JSON into Jira DR-DRILL-BVR ticket.

## Step 10: Kopia restore (if application data corrupted)

If the voter data volume is corrupted (not just PG failover):

```bash
# List Kopia snapshots for T1
KOPIA_PASSWORD=$(vault read -field=plaintext transit/eel/kopia-t1-key | base64 -d)
kopia --config-file=platform/kopia/kopia-t1.json snapshot list /data/t1

# Restore to a scratch path
kopia --config-file=platform/kopia/kopia-t1.json snapshot restore \
  <snapshot-id> /data/t1-restored

# Verify row count matches expected, then promote
```

## Step 11: Post-incident

1. pgBackRest integrity check: `pgbackrest --stanza=voter-db check`
2. Verify Kopia backup schedule ran: check MinIO bucket `kopia-t1-primary` for today's snapshot
3. Update DR drill log, close ticket, retrospective within 48 h
