# Transparency log witness cross-check

## Purpose

Run a routine or triggered consistency check of the transparency log against its witnesses. The log is append-only and signed; witnesses independently verify the signed tree heads (STHs) and cosign them. A cross-check confirms the log has not forked, that all witnesses agree, and that inclusion and consistency proofs hold.

## Prerequisites

- `verifier-rs` CLI available on the ops jump host.
- Access to the transparency log API and the witness cosign endpoints.
- An incident ticket open as `<TICKET>` (routine checks are pre-approved).
- IAB witness client available for the cross-check.

## Commands

### Step 1: confirm the log is healthy

```bash
# check the STH cadence and the latest signed tree head:
curl -sf http://transparency-svc.eel-system.svc.cluster.local:9001/admin/sth
# confirm the STH is fresh (within the 1s/60s/5m cadence)
```

### Step 2: run the witness cross-check

```bash
# verify each witness's cosignature against the current STH:
verifier-rs witness-check --log transparency-svc.eel-system.svc.cluster.local:9000
# confirm all witnesses agree on the current tree head
```

### Step 3: verify inclusion proofs

```bash
# sample recent leaves and verify their inclusion proofs:
verifier-rs inclusion --log transparency-svc.eel-system.svc.cluster.local:9000 \
  --leaf <LEAF_ID> --sth <STH>
# confirm each sampled leaf is included in the current tree
```

### Step 4: verify consistency

```bash
# verify the log is append-only between two STHs:
verifier-rs consistency --log transparency-svc.eel-system.svc.cluster.local:9000 \
  --from <STH_OLD> --to <STH_NEW>
# confirm the older tree is a prefix of the newer tree
```

### Step 5: check for witness divergence

```bash
# if any witness disagrees, the log may have forked:
# follow witness-divergence-response.md
# if all witnesses agree, the log is consistent
```

### Step 6: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.tlog-witness-cross-check.completed.v1\",\"payload\":{\"ticket\":\"<TICKET>\",\"witnesses_agree\":\"<N>\",\"leaves_verified\":\"<N>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. All witnesses cosign the current STH.
2. Sampled leaves have valid inclusion proofs.
3. Consistency holds between consecutive STHs.
4. No witness divergence detected.
5. Transparency log contains the cross-check-completed leaf.

## Rollback procedure

A cross-check is read-only; there is nothing to roll back. If divergence is detected, do not modify the log; follow `witness-divergence-response.md` to determine which view is authoritative and restore consistency.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- Security Officer (contact via encrypted comms) if divergence is detected
- IAB liaison (contact via war room bridge) for witness coordination

## Estimated duration

15 to 30 minutes.

## Chaos-drill reference

This runbook is exercised by `chaos/scenarios/eel-tlog-fork.yaml` (log split-view). Related: `docs/runbooks/witness-divergence-response.md` and `docs/runbooks/hsm-loss.md` (STH signing pause).
