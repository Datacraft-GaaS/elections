# Warranty operations: KEBS and OEM

## Overview

eel devices (BVI kits, BVR kits, RTS kits, staff mobiles) carry two types of warranty coverage:

1. **KEBS conformity:** Kenya Bureau of Standards conformity certificate required for all procured devices. Non-conformant devices may not be deployed.
2. **OEM warranty:** manufacturer warranty covering hardware defects; coverage period and contact vary by vendor and device model.

The authoritative coverage matrix is in `coverage-matrix.yaml`. All warranty claims are tracked via `inventory-svc` using the device serial number.

## Warranty claim workflow

```
Hardware failure detected -> Field staff reports -> inventory-svc custody event -> Support team assesses -> OEM claim submitted -> RMA issued -> Replacement received -> Re-provisioned
```

See `rma-workflow.md` for the full swimlane.

## KEBS conformity tracking

All devices must have a valid KEBS conformity certificate at procurement. The certificate number and expiry are recorded in `inventory-svc` at the `device.received` event.

```bash
# check KEBS status for a device:
curl -sf http://inventory-svc.eel-system.svc.cluster.local:8080/devices/<SERIAL>/kebs \
  | python3 -m json.tool
# look for "kebs_cert_status": "valid"

# list devices with expiring KEBS certs:
curl -sf "http://inventory-svc.eel-system.svc.cluster.local:8080/devices/kebs-expiring?before=<DATE>" \
  | python3 -m json.tool
```

## Decommission tie-in

When a device is decommissioned (end of service life), the warranty record is closed in `inventory-svc`:

```bash
curl -sf -X POST http://inventory-svc.eel-system.svc.cluster.local:8080/devices/<SERIAL>/warranty/close \
  -H "Content-Type: application/json" \
  -d '{"reason":"decommissioned","ticket":"<TICKET>"}'
```

## Files

| File | Description |
|------|-------------|
| `rma-workflow.md` | Swimlane RMA procedure |
| `coverage-matrix.yaml` | Vendor x device x coverage period x contact |
