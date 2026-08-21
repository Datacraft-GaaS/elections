# Device lifecycle: provision, wipe, decommission, RMA

## Overview

Every eel device (kit or staff mobile) passes through a defined lifecycle. Each stage emits a custody event to `inventory-svc`, which maintains the authoritative device register. HR integration (M55) triggers onboarding and offboarding stages automatically.

## Lifecycle stages

| Stage | Trigger | Actions | inventory-svc event |
|-------|---------|---------|---------------------|
| Procurement | Purchase order approved | Receive device; record serial number and IMEI | `device.received` |
| Enrolment | Kit assigned to deployment | MDM enrol; SPIRE SVID issued; attestation verified | `device.enrolled` |
| Deployment | Field assignment created | Physical handover to officer; officer acknowledges | `device.deployed` |
| Active | Operational period | Periodic check-ins; MDM compliance checks | `device.checkin` |
| Retrieval | End of field period | Physical return; MDM compliance report generated | `device.retrieved` |
| Redeployment | New field assignment | Re-attest; re-enrol if policy changed | `device.redeployed` |
| Decommission | End of service life or RMA | MDM wipe; SVID revoked; serial number retired | `device.decommissioned` |
| RMA | Hardware failure or damage | RMA initiated with OEM; see warranty runbook | `device.rma` |

## Provision procedure

```bash
# 1. record device in inventory:
curl -sf -X POST http://inventory-svc.eel-system.svc.cluster.local:8080/devices \
  -H "Content-Type: application/json" \
  -d "{\"serial\":\"<SERIAL>\",\"imei\":\"<IMEI>\",\"model\":\"<MODEL>\",\"category\":\"<kit|mobile|staff-ios>\"}"

# 2. enrol in MDM:
#    for Android Enterprise: use the MDM console or zero-touch enrolment
#    for Apple ABM: device appears in ABM automatically after purchase

# 3. assign MDM profile:
curl -sf -X POST https://mdm.eel.io/api/v1/devices/<DEVICE_ID>/profile \
  -H "Authorization: Bearer <MDM_API_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{"profile":"eel-kit-profile","ticket":"<TICKET>"}'

# 4. verify attestation:
curl -sf http://kit-mgmt-svc.eel-system.svc.cluster.local:9001/kits/<SERIAL>/attestation
# must return "status":"attested"
```

## Wipe procedure (decommission or RMA)

Wipe requires dual-officer sign-off for kit devices.

```bash
# 1. confirm device is not in active field deployment:
curl -sf http://inventory-svc.eel-system.svc.cluster.local:8080/devices/<SERIAL> \
  | python3 -m json.tool | grep '"stage"'

# 2. revoke SPIRE SVID:
kubectl exec -n eel-system deploy/spire-server -- \
  spire-server entry delete -entryID <SPIRE_ENTRY_ID>

# 3. issue MDM wipe command:
curl -sf -X POST https://mdm.eel.io/api/v1/devices/<DEVICE_ID>/commands \
  -H "Authorization: Bearer <MDM_API_TOKEN>" \
  -H "Content-Type: application/json" \
  -d "{\"command\":\"WIPE\",\"officer_a\":\"<ID_A>\",\"officer_b\":\"<ID_B>\",\"ticket\":\"<TICKET>\"}"

# 4. emit decommission custody event:
curl -sf -X POST http://inventory-svc.eel-system.svc.cluster.local:8080/devices/<SERIAL>/events \
  -H "Content-Type: application/json" \
  -d "{\"event\":\"device.decommissioned\",\"officer\":\"<OFFICER_ID>\",\"ticket\":\"<TICKET>\"}"
```

## HR integration (M55)

When an officer's HR record is updated (role change, offboarding), the HR integration emits an event that triggers:

1. For offboarding: device retrieval task created in `support-admin-svc`; MDM profile set to retrieval mode (disables eel apps, displays return instruction).
2. For role change: device redeployment flow triggered; new profile applied.

## RMA workflow

See `operations/warranty/rma-workflow.md` for the full RMA swimlane. The MDM step in an RMA is:

1. Issue wipe command (above) before shipping the device to the OEM.
2. Record RMA ticket number in `inventory-svc`.
3. On return, treat the replacement device as a new procurement.

## References

- `operations/warranty/README.md`: KEBS and OEM warranty
- `operations/dispatch/README.md`: field deployment logistics
- `docs/runbooks/kit-mass-revocation.md`: emergency revocation
- `docs/runbooks/stolen-kit-honey-mode.md`: stolen device response
