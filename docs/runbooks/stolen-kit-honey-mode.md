# Stolen kit: honey mode activation

## Purpose

Activate honey mode on a BVR or BVI kit reported stolen or missing. In honey mode the kit operates apparently normally but all actions are silently logged to `transparency-svc` without alerting the holder. This enables evidence capture for law enforcement.

## Prerequisites

- The kit serial number is known and recorded (`<KIT_SERIAL>`).
- Access to the `kit-mgmt` API (`http://kit-mgmt-svc.eel-system.svc.cluster.local:9001`).
- An incident ticket open as `<TICKET>`.
- Security Officer approval on record in the ticket.
- Do NOT revoke the kit SVID before honey mode is confirmed active; revocation would alert the holder.

## Commands

### Step 1: confirm kit serial and current status

```bash
curl -sf http://kit-mgmt-svc.eel-system.svc.cluster.local:9001/kits/<KIT_SERIAL> \
  | python3 -m json.tool
# confirm kit_status is "active" and last_seen timestamp is recent
```

### Step 2: activate honey mode via kit-mgmt API

```bash
curl -sf -X POST \
  http://kit-mgmt-svc.eel-system.svc.cluster.local:9001/kits/<KIT_SERIAL>/honey-mode \
  -H "Content-Type: application/json" \
  -d "{\"enabled\":true,\"reason\":\"stolen\",\"ticket\":\"<TICKET>\",\"officer\":\"<OFFICER_ID>\"}"
# expected response: {"honey_mode":"active","kit_serial":"<KIT_SERIAL>"}
```

### Step 3: verify transparency log entries are flowing

```bash
# monitor tlog for the kit's events:
watch -n 10 'grpcurl -plaintext \
  -d "{\"filter\":{\"source_id\":\"<KIT_SERIAL>\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogQuery/Stream | tail -5'
# events should appear within the kit's normal activity cycle
```

### Step 4: verify the kit holder receives no alerts

```bash
# check that no revocation or alert event has been emitted for this kit:
grpcurl -plaintext \
  -d "{\"filter\":{\"source_id\":\"<KIT_SERIAL>\",\"event_type_prefix\":\"kit.alert\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogQuery/Stream
# must return no results
```

### Step 5: emit operational tlog entry (restricted access)

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.kit.honey-mode.activated.v1\",\"payload\":{\"kit_serial\":\"<KIT_SERIAL>\",\"incident_ticket\":\"<TICKET>\",\"officer\":\"<OFFICER_ID>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
# this leaf is in the restricted evidence namespace; normal tlog viewers cannot see it
```

### Step 6: coordinate law enforcement handoff

```bash
# generate evidence package from tlog:
eel-evidence-export \
  --kit-serial <KIT_SERIAL> \
  --since <HONEY_START_ISO8601> \
  --out /tmp/evidence-<KIT_SERIAL>-<TICKET>.tar.gpg \
  --recipient <LEA_GPG_KEY_ID>
# deliver package via secure courier; record handoff in incident ticket
```

## Verification steps

1. `kit-mgmt` API returns `"honey_mode":"active"` for the kit.
2. Tlog shows kit events flowing after activation.
3. No alert events appear in the kit's tlog stream.
4. Evidence export file is non-empty and GPG-encrypted.

## Rollback procedure

To deactivate honey mode (e.g., kit is recovered or operation is concluded):

```bash
curl -sf -X POST \
  http://kit-mgmt-svc.eel-system.svc.cluster.local:9001/kits/<KIT_SERIAL>/honey-mode \
  -H "Content-Type: application/json" \
  -d '{"enabled":false,"ticket":"<TICKET>","officer":"<OFFICER_ID>"}'
```

After deactivation, decide whether to revoke the kit SVID (if it remains unrecovered) or return it to normal service (if recovered). Consult Security Officer before reactivating a recovered stolen kit.

## Escalation contacts

- Security Officer (contact via encrypted comms)
- Site Reliability Lead (on-call via PagerDuty rotation)
- Legal and Compliance (contact details in the incident ticket template)

## Estimated duration

5 to 10 minutes to activate. Evidence capture duration depends on law enforcement timeline.

## Chaos-drill reference

This runbook references: `chaos/scenarios/eel-svid-revocation.yaml` for the post-honey-mode revocation phase.

Honey mode itself has no dedicated chaos scenario; it is exercised in tabletop drills only (no automated simulation due to evidence integrity requirements).
