# DR failover: primary site to mirror site

## Purpose

Full data-centre failover from the primary eel site to the mirror site when the primary is unrecoverable in the short term. Uses the `tools/dr-failover` binary to orchestrate the sequence atomically.

RTO target: 15 minutes from decision to mirror accepting writes.
RPO target: 30 seconds (Kafka MirrorMaker2 lag budget).

## Prerequisites

- `tools/dr-failover` binary built and present on the ops jump host (`cargo build --release -p dr-failover`).
- Mirror site Patroni cluster is healthy (`patronictl -c /etc/patroni/mirror.yaml list` shows all replicas in sync).
- MirrorMaker2 replication lag is below 30 seconds: verify via `kafka-consumer-groups.sh` on mirror brokers.
- An incident ticket is open. Record the ticket ID in every command below where `<TICKET>` appears.
- Two authorised officers present (dual-person integrity for production failover).
- VPN or direct network access to both primary and mirror control planes.

## Commands

### Step 1: confirm primary is unreachable

```bash
dr-failover preflight --site primary --ticket <TICKET>
# exits non-zero if primary is still responding; review before proceeding
```

### Step 2: freeze writes on primary (if partially reachable)

```bash
dr-failover freeze-primary --site primary --ticket <TICKET>
# sets the eel.write-gate ConfigMap to "frozen" in eel-system namespace
# if primary API server is down, skip to step 3
```

### Step 3: verify mirror replication state

```bash
dr-failover check-mirror --ticket <TICKET>
# prints MirrorMaker2 consumer group lag per topic
# abort if lag > 30s and data loss is unacceptable
```

### Step 4: promote mirror Patroni primary

```bash
patronictl -c /etc/patroni/mirror.yaml failover eel-pg --master auto --force
patronictl -c /etc/patroni/mirror.yaml list
# confirm one node shows "Leader"
```

### Step 5: update DNS and anycast

```bash
dr-failover update-dns --target mirror --ticket <TICKET>
# updates eel.io A/AAAA records and withdraws primary BGP prefix
# verify with: dig +short api.eel.io
```

### Step 6: switch MirrorMaker2 direction

```bash
dr-failover flip-mm2 --source mirror --destination primary-standby --ticket <TICKET>
# reconfigures MirrorMaker2 to replicate from mirror back to primary once primary recovers
```

### Step 7: verify Kafka consumer groups on mirror

```bash
kafka-consumer-groups.sh --bootstrap-server mirror-kafka:9092 \
  --list | xargs -I{} kafka-consumer-groups.sh \
  --bootstrap-server mirror-kafka:9092 --describe --group {}
# confirm lag is 0 for all critical groups
```

### Step 8: run smoke tests

```bash
dr-failover smoke-test --site mirror --ticket <TICKET>
# exercises bvr-svc, rts-svc, and transparency-svc health endpoints
```

### Step 9: emit tlog failover entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.dr-failover.primary-to-mirror.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"rpo_lag_seconds\":\"<LAG>\",\"officer_a\":\"<ID_A>\",\"officer_b\":\"<ID_B>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. `dr-failover status --site mirror` returns `ACTIVE`.
2. `dig +short api.eel.io` resolves to a mirror site IP.
3. All critical Kafka consumer groups show zero lag on mirror brokers.
4. Smoke test suite passes (step 8 output shows `ALL_PASS`).
5. Transparency log shows the failover leaf (query `transparency-svc` for event type `meta-events.dr-failover.primary-to-mirror.v1`).

## Rollback procedure

Rollback is only possible if the primary site recovers. Follow `dr-failback.md` for the controlled return sequence. Do not attempt to restore primary as active while mirror is serving live traffic without completing the full failback runbook.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- DR Coordinator (contact via war room bridge)
- Database Lead for Patroni issues (on-call via PagerDuty rotation)

## Estimated duration

10 to 15 minutes from decision to mirror accepting writes.

## Chaos-drill reference

This runbook is exercised by: `chaos/scenarios/eel-site-partition-primary-mirror.yaml`

Run the drill monthly under `docs/runbooks/chaos-monthly-drill.md` supervision.
