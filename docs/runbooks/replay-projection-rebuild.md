# Replay projection rebuild

## Purpose

Rebuild a read-model projection that has drifted or been corrupted. Projections are derived from the event log; a rebuild replays the source events from a safe offset to reconstruct the projection deterministically. The goal is to restore the projection to a state consistent with the event log without data loss.

## Prerequisites

- The projection service and its event consumer available.
- The event log (outbox/Kafka) intact for the projection's source topics.
- An incident ticket open as `<TICKET>`.
- The projection's last-known-good offset recorded, if available.

## Commands

### Step 1: confirm the projection is corrupt

```bash
# compare the projection's materialised state against the event log:
# a checksum or row-count mismatch indicates drift
# confirm the source topics are healthy and lag is bounded
```

### Step 2: determine a safe replay offset

```bash
# choose the offset to replay from:
# - the last-known-good offset, or
# - the earliest retained offset for a full rebuild
# record the chosen offset as <OFFSET>
```

### Step 3: stop the projection consumer

```bash
# pause the consumer so it does not write during the rebuild:
kubectl scale deploy/<projection-svc> --replicas=0 -n eel-system
# confirm the consumer group is idle
```

### Step 4: truncate the projection

```bash
# drop the projection's materialised tables/state:
psql -d eel -c "TRUNCATE <PROJECTION_TABLE>;"
# or delete the projection's index/state store
```

### Step 5: rebuild from the event log

```bash
# run the projection rebuild, replaying from <OFFSET>:
eel-projection-rebuild --projection <NAME> --from <OFFSET> --ticket <TICKET>
# the rebuild applies each event idempotently and writes a rebuild event to the outbox
```

### Step 6: verify the rebuild

```bash
# compare the rebuilt projection against the event log:
# row count and checksum must match the source
# confirm consumer lag returns to 0
```

### Step 7: resume the consumer

```bash
kubectl scale deploy/<projection-svc> --replicas=1 -n eel-system
# confirm the consumer resumes at the correct offset with no gap
```

### Step 8: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.projection-rebuild.completed.v1\",\"payload\":{\"ticket\":\"<TICKET>\",\"projection\":\"<NAME>\",\"from_offset\":\"<OFFSET>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. The rebuilt projection matches the event log (row count and checksum).
2. Consumer lag returns to 0.
3. No events were lost or duplicated during the rebuild.
4. The rebuild event is present in the transparency log.
5. Downstream readers observe the corrected projection.

## Rollback procedure

A projection rebuild is idempotent and reversible: re-run the rebuild from the same or an earlier offset to reconstruct any state. If the rebuild produced a wrong result, truncate and rebuild again from a corrected offset. The event log is the source of truth and is never modified by a rebuild.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- Service owner for the projection (contact via war room bridge)
- Database Lead (on-call via PagerDuty rotation) if the projection is large

## Estimated duration

15 to 60 minutes depending on projection size and replay volume.

## Chaos-drill reference

This runbook is exercised by `chaos/scenarios/eel-projection-drift.yaml` (projection corruption from a partial write). Related: `docs/runbooks/event-replay.md` (event replay) and `docs/runbooks/consumer-lag-response.md` (lag).
