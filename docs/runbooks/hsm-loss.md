# HSM loss: sequencer degradation and backup HSM activation

## Purpose

Respond to HSM unavailability by placing the sequencer into degraded (queued countersign) mode, activating the backup HSM, and draining the countersign queue without dropping events. This runbook assumes a single HSM failure; for full HSM compromise, use `break-glass.md` instead.

## Prerequisites

- Access to the sequencer service admin API (`http://sequencer-svc.eel-system.svc.cluster.local:9001/admin`).
- Backup HSM is pre-provisioned and network-reachable (configured in `platform/hsm/backup.conf`).
- HSM admin credentials stored in OpenBao at `kv/eel/prod/hsm/backup-admin`.
- An incident ticket open as `<TICKET>`.

## Commands

### Step 1: confirm HSM is offline

```bash
curl -sf http://sequencer-svc.eel-system.svc.cluster.local:9001/admin/hsm-health \
  | grep '"status"'
# expected: "status":"degraded" or "status":"offline"
kubectl logs -n eel-system deploy/sequencer-svc --tail=30 | grep -i hsm
```

### Step 2: enable sequencer degraded mode

```bash
# countersigns are queued locally; events continue to flow without HSM signatures
curl -sf -X POST \
  http://sequencer-svc.eel-system.svc.cluster.local:9001/admin/degraded-mode \
  -H "Content-Type: application/json" \
  -d '{"enabled":true,"reason":"HSM offline","ticket":"<TICKET>"}'
# confirm:
curl -sf http://sequencer-svc.eel-system.svc.cluster.local:9001/admin/status \
  | grep '"mode"'
# expected: "mode":"degraded"
```

### Step 3: retrieve backup HSM credentials

```bash
export VAULT_ADDR=https://vault.eel-system.svc.cluster.local:8200
vault kv get -field=pin kv/eel/prod/hsm/backup-admin
```

### Step 4: activate backup HSM

```bash
# initialise the backup HSM connection (adjust device path as needed):
eel-hsm-admin --device /dev/hsm-backup \
  --config platform/hsm/backup.conf \
  --pin "$(vault kv get -field=pin kv/eel/prod/hsm/backup-admin)" \
  activate --ticket <TICKET>
# verify backup HSM is reachable:
eel-hsm-admin --device /dev/hsm-backup health
```

### Step 5: reconfigure sequencer to use backup HSM

```bash
kubectl set env deployment/sequencer-svc \
  HSM_DEVICE_PATH=/dev/hsm-backup \
  HSM_CONFIG=platform/hsm/backup.conf \
  -n eel-system
kubectl rollout status deployment/sequencer-svc -n eel-system
```

### Step 6: disable degraded mode and drain countersign queue

```bash
curl -sf -X POST \
  http://sequencer-svc.eel-system.svc.cluster.local:9001/admin/degraded-mode \
  -H "Content-Type: application/json" \
  -d '{"enabled":false,"ticket":"<TICKET>"}'
# drain: sequencer will countersign queued events automatically
# monitor queue depth:
watch -n 3 'curl -sf \
  http://sequencer-svc.eel-system.svc.cluster.local:9001/admin/queue-depth'
# wait for depth to reach 0
```

### Step 7: verify no events dropped

```bash
# compare event counts in transparency log vs sequencer emit counter:
curl -sf http://sequencer-svc.eel-system.svc.cluster.local:9001/metrics \
  | grep eel_sequencer_events_total
# compare with tlog leaf count for the same time window
```

### Step 8: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.hsm-loss.backup-activated.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. Sequencer mode returns `"mode":"normal"` after degraded mode disabled.
2. Countersign queue depth is 0.
3. Event emit rate (Prometheus metric `eel_sequencer_events_total`) resumes normal rate.
4. No events in dead-letter topics (`eel.dlq.*`).
5. Transparency log contains the hsm-loss leaf.

## Rollback procedure

If the backup HSM cannot be activated, keep degraded mode enabled and escalate immediately. The sequencer will continue to queue countersigns indefinitely (bounded by local disk). The queue will be drained once any HSM is available. Do not restart the sequencer pod while degraded mode is active, as the queue is in-memory; a restart will drop queued countersigns.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- Security Officer (contact via encrypted comms) for HSM key management
- HSM Vendor Support (contact details in `platform/hsm/vendor-contacts.txt`)

## Estimated duration

10 to 20 minutes to activate backup HSM and drain queue.

## Chaos-drill reference

This runbook is exercised by: `chaos/scenarios/eel-hsm-unavailable.yaml` and `chaos/scenarios/eel-sequencer-stall.yaml`
