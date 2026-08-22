# HSM key ceremony: scheduled key generation and rotation

## Purpose

Generate or rotate HSM keys under a witnessed, audited ceremony. The ceremony covers the key roles used by the platform: `sth` (signed tree head), `declaration`, `kit-ca`, `credential-ca`, `code-signing`, and `cycle-KEK`. Every ceremony step is recorded in the transparency log as a `meta-events` leaf. For an HSM that is already offline, use `hsm-loss.md`; for full compromise, use `break-glass.md`.

## Prerequisites

- HSM admin credentials stored in OpenBao at `kv/eel/prod/hsm/admin`.
- The `eel-hsm-admin` CLI installed on the ceremony workstation.
- A quorum of key custodians present (M-of-N, default 3-of-5) as `<CUSTODIAN_1..N>`.
- An incident ticket open as `<TICKET>` (ceremonies are pre-approved).
- A blank ceremony log sheet to record each step and its tlog leaf hash.

## Commands

### Step 1: confirm HSM health and current key inventory

```bash
eel-hsm-admin --device /dev/hsm-primary health
eel-hsm-admin --device /dev/hsm-primary list-keys
# record the current key IDs and their roles
```

### Step 2: open the ceremony

```bash
eel-hsm-admin --device /dev/hsm-primary \
  --pin "$(vault kv get -field=pin kv/eel/prod/hsm/admin)" \
  ceremony open --ticket <TICKET> --quorum 3 --custodians <CUSTODIAN_1> <CUSTODIAN_2> <CUSTODIAN_3>
# record the ceremony ID
```

### Step 3: generate or rotate each key role

```bash
for role in sth declaration kit-ca credential-ca code-signing cycle-KEK; do
  eel-hsm-admin --device /dev/hsm-primary \
    ceremony rotate --role "$role" --ceremony <CEREMONY_ID>
  # record the new key ID and the tlog leaf hash returned
done
```

### Step 4: verify each new key

```bash
for role in sth declaration kit-ca credential-ca code-signing cycle-KEK; do
  eel-hsm-admin --device /dev/hsm-primary \
    verify --role "$role" --ceremony <CEREMONY_ID>
done
# each verify must return OK
```

### Step 5: re-wrap dependent secrets

```bash
# if cycle-KEK rotated, re-wrap OpenBao transit keys that depend on it:
vault transit rewrap eel/cycle-kek
# confirm dependent services pick up the new key material:
kubectl rollout restart deployment/sequencer-svc -n eel-system
```

### Step 6: close the ceremony

```bash
eel-hsm-admin --device /dev/hsm-primary \
  ceremony close --ceremony <CEREMONY_ID> --ticket <TICKET>
```

### Step 7: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.hsm-key-ceremony.completed.v1\",\"payload\":{\"ticket\":\"<TICKET>\",\"ceremony_id\":\"<CEREMONY_ID>\",\"roles\":\"<ROLES>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. Every key role verifies OK after rotation.
2. Dependent secrets are re-wrapped and services restarted cleanly.
3. The ceremony log sheet records each step with its tlog leaf hash.
4. Transparency log contains the ceremony-completed leaf.
5. No key was generated or rotated outside the ceremony (audit the HSM log).

## Rollback procedure

If a rotated key fails verification, do not close the ceremony. Re-import the previous key from the ceremony backup (stored in OpenBao at `kv/eel/prod/hsm/backup`) and re-verify. If the HSM becomes unusable mid-ceremony, follow `hsm-loss.md` to activate the backup HSM, then resume the ceremony against it. Never leave a ceremony open unattended.

## Escalation contacts

- Security Officer (contact via encrypted comms)
- HSM Vendor Support (contact details in `platform/hsm/vendor-contacts.txt`)
- Site Reliability Lead (on-call via PagerDuty rotation)

## Estimated duration

2 to 4 hours.

## Chaos-drill reference

This is a scheduled ceremony, not a chaos scenario. It is related to `docs/runbooks/hsm-loss.md` (backup HSM activation) and `docs/runbooks/break-glass.md` (compromise response).
