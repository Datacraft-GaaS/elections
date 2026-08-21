# Witness divergence response

## Purpose

Respond to a split-view alert where two or more transparency log witnesses disagree on the current tree head. A divergence indicates either a network partition between witnesses or a Byzantine fault in the log. New leaf submissions are paused until the authoritative view is confirmed.

## Prerequisites

- Access to the `transparency-svc` admin API.
- Contact details for all registered witness operators (stored in `platform/tlog/witnesses.yaml`).
- An incident ticket open as `<TICKET>`.
- Security Officer notified; this is a high-severity integrity event.

## Commands

### Step 1: detect divergence details

```bash
# the alert fires from transparency-svc; query the divergence report:
curl -sf http://transparency-svc.eel-system.svc.cluster.local:9001/admin/witness-status \
  | python3 -m json.tool
# output shows each witness's latest STH (signed tree head) and tree size
```

### Step 2: pause new leaf submissions

```bash
curl -sf -X POST \
  http://transparency-svc.eel-system.svc.cluster.local:9001/admin/submission-gate \
  -H "Content-Type: application/json" \
  -d '{"state":"paused","reason":"witness-divergence","ticket":"<TICKET>"}'
# confirm:
curl -sf http://transparency-svc.eel-system.svc.cluster.local:9001/admin/submission-gate
# must return "state":"paused"
```

### Step 3: identify the authoritative witness

```bash
# compare tree sizes and timestamps:
curl -sf http://transparency-svc.eel-system.svc.cluster.local:9001/admin/witness-status \
  | python3 -c "
import sys, json
data = json.load(sys.stdin)
for w in data['witnesses']:
    print(w['id'], w['tree_size'], w['sth_timestamp'])
"
# the witness with the highest tree size and most recent valid STH signature is authoritative
# verify the STH signature independently:
eel-tlog-verify --sth <BASE64_STH> --pub-key platform/tlog/witness-<ID>.pub
```

### Step 4: diagnose the diverging witness

```bash
# check connectivity to the diverging witness:
curl -sf https://witness-<ID>.eel.io/health || echo "WITNESS UNREACHABLE"
# check its tlog sync lag:
curl -sf https://witness-<ID>.eel.io/api/v0/get-tree-head-to-sign
# compare tree size with authoritative witness
```

### Step 5: reconcile or escalate

```bash
# if divergence is due to network partition (witness is simply behind):
# wait for it to sync, monitoring with:
watch -n 10 'curl -sf http://transparency-svc.eel-system.svc.cluster.local:9001/admin/witness-status \
  | python3 -m json.tool | grep tree_size'
# if divergence persists > 10 minutes or tree hashes conflict (not just sizes):
# escalate to the witness operator and the log operator immediately
# do not resume submissions until all witnesses agree
```

### Step 6: resume submissions after quorum

```bash
# confirm all witnesses agree on tree head:
curl -sf http://transparency-svc.eel-system.svc.cluster.local:9001/admin/witness-status \
  | python3 -c "
import sys, json
data = json.load(sys.stdin)
sizes = set(w['tree_size'] for w in data['witnesses'])
print('QUORUM OK' if len(sizes) == 1 else 'DIVERGENCE: ' + str(sizes))
"
# only resume if output is QUORUM OK:
curl -sf -X POST \
  http://transparency-svc.eel-system.svc.cluster.local:9001/admin/submission-gate \
  -H "Content-Type: application/json" \
  -d '{"state":"open","ticket":"<TICKET>"}'
```

### Step 7: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.witness-divergence.resolved.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"duration_minutes\":\"<MINUTES>\",\"root_cause\":\"<CAUSE>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. All witnesses report identical tree size and STH hash.
2. Submission gate returns `"state":"open"`.
3. New leaves are being accepted (check `transparency-svc` metrics: `eel_tlog_leaves_total`).
4. No leaves were dropped during the pause (compare pre-pause and post-resume sequence numbers).

## Rollback procedure

If divergence cannot be resolved (Byzantine fault confirmed), do not resume submissions. Escalate to the log operator and Security Officer. The log may need to be forked or the Byzantine witness removed from the quorum set. This is a manual process coordinated with all witness operators.

## Escalation contacts

- Security Officer (contact via encrypted comms)
- Transparency Log Operator (contact details in `platform/tlog/witnesses.yaml`)
- Site Reliability Lead (on-call via PagerDuty rotation)

## Estimated duration

15 to 60 minutes depending on whether the divergence is a partition lag or a Byzantine fault.

## Chaos-drill reference

This runbook is exercised by the partition phase of: `chaos/scenarios/eel-site-partition-primary-mirror.yaml`

A dedicated witness-divergence chaos scenario is planned; reference this runbook from it once created.
