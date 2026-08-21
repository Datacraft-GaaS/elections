# DR Failover Runbook: rts-svc (Results Transmission Service)

SLO: RTO <= 30 min, RPO <= 5 min (ADR-0022, spec-binding)

## Prerequisites

- Two-human quorum in room (election-day rule) or on comms channel
- `kubectl` context set to the active site
- `eel-dr-failover` binary available: `tools/dr-failover/target/release/eel-dr-failover`
- Access to Patroni CLI on a T1 database node

## Step 0: Declare incident

1. Page NOC + SOC on-call: `#eel-noc` Slack, PagerDuty "EEL DR Failover"
2. Record incident start time (UTC): `START=$(date -u +%Y-%m-%dT%H:%M:%SZ)`
3. Open Jira DR-DRILL-{YYYY-MM-DD} ticket

## Step 1: Confirm failure scope

```bash
# Are rts-svc pods healthy on the primary site?
kubectl -n eel-prod get pods -l app=rts-svc

# Is the T1 primary database reachable?
kubectl -n eel-prod exec deploy/rts-svc -- \
  psql "$DATABASE_URL" -c "SELECT 1;" 2>&1

# What is the current WAL lag on the mirror replica?
kubectl -n eel-prod exec deploy/rts-svc -- \
  psql "$MIRROR_DATABASE_URL" -c \
  "SELECT now() - pg_last_xact_replay_timestamp() AS lag;"
```

Decision gate: if lag > 5 min, record RPO breach and escalate to incident commander.

## Step 2: Review the failover plan

```bash
eel-dr-failover plan --service rts-svc
```

Or for full-site failover:

```bash
eel-dr-failover plan
```

## Step 3: Dry-run failover

```bash
eel-dr-failover execute --service rts-svc --dry-run
```

Confirm the steps look correct with both operators.

## Step 4: Promote Patroni replica (T1 primary kill scenario)

```bash
# Identify mirror-site replica
patronictl -c /etc/patroni/patroni.yml list

# Failover
patronictl -c /etc/patroni/patroni.yml failover \
  --master <current-primary-pod> \
  --candidate <mirror-replica-pod> \
  --force

# Wait for promotion (target: < 60 s)
watch -n 2 patronictl -c /etc/patroni/patroni.yml list
```

Verification:
```bash
# Exactly one primary, on the mirror site
patronictl list | grep -c " Leader"  # must be 1
psql "$MIRROR_DATABASE_URL" -c "SELECT pg_is_in_recovery();"  # must return 'f'
```

## Step 5: Update service endpoints

```bash
# Patch the rts-svc Kubernetes Service to point at mirror-site PG
kubectl -n eel-prod patch configmap rts-svc-config \
  --patch '{"data":{"DATABASE_HOST":"postgres-t1-mirror.eel-prod.svc.cluster.local"}}'

# Restart rts-svc pods to pick up new DB host
kubectl -n eel-prod rollout restart deploy/rts-svc
kubectl -n eel-prod rollout status deploy/rts-svc --timeout=5m
```

## Step 6: Verify write path

```bash
# Health check
curl -sf https://rts.eel.internal/healthz | jq .

# Probe write (idempotent probe endpoint)
curl -sf -X POST https://rts.eel.internal/api/v1/results/probe \
  -H "Authorization: Bearer $PROBE_TOKEN" | jq .status
# Expected: "ok"

# Check P99 write latency in Prometheus
curl -sg 'http://prometheus.monitoring.svc.cluster.local:9090/api/v1/query?query=histogram_quantile(0.99,rate(rts_write_duration_seconds_bucket[5m]))' \
  | jq '.data.result[0].value[1]'
# Must be < 5.0 (seconds)
```

## Step 7: Verify Kafka replication lag

```bash
kafka-consumer-groups.sh \
  --bootstrap-server kafka-mirror.eel-prod.svc.cluster.local:9092 \
  --describe \
  --group rts-outbox

# All partitions must show LAG < 10 000 within 2 min of failover
```

## Step 8: Record RTO and RPO

```bash
END=$(date -u +%Y-%m-%dT%H:%M:%SZ)
echo "Failover window: $START to $END"

eel-dr-failover report \
  --drill "rts-svc-$(date +%Y%m%d)" \
  --check-svc rts-svc \
  --format json | tee /tmp/dr-report-rts-$(date +%Y%m%d).json
```

Paste report JSON into the Jira DR-DRILL ticket.

## Step 9: Split-brain triage (if suspected)

If Patroni shows two primaries:

1. STOP: do not accept writes on either site
2. Identify which primary has the higher LSN:
   ```bash
   psql -h <primary-A> -c "SELECT pg_current_wal_lsn();"
   psql -h <primary-B> -c "SELECT pg_current_wal_lsn();"
   ```
3. Fence the lower-LSN node: `patronictl pause <node>` then shut it down
4. Reconcile diverged writes with the incident commander
5. Never merge conflicting results data -- treat as a security incident

## Step 10: Post-incident

1. Run pgBackRest integrity check:
   ```bash
   pgbackrest --stanza=voter-db check
   ```
2. Verify Velero backup completed overnight: `velero backup get | head -5`
3. Update DR drill log and close Jira ticket
4. Retrospective within 48 h -- RTO/RPO actuals vs SLO in the retro doc
