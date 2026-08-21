# RMA workflow

## Swimlane

```
Field Staff -> Field Support -> OEM/Vendor -> eel Inventory -> Field Support -> Field Staff
```

### Phase 1: failure report (Field Staff)

1. Officer reports hardware failure via support mobile app or helpdesk.
2. `field-mobile-svc` creates a Zammad ticket with category "Kit Hardware Failure".
3. `inventory-svc` receives a `device.fault_reported` custody event.

### Phase 2: assessment (Field Support)

```bash
# retrieve device details:
curl -sf http://inventory-svc.eel-system.svc.cluster.local:8080/devices/<SERIAL> \
  | python3 -m json.tool

# check warranty coverage:
curl -sf http://inventory-svc.eel-system.svc.cluster.local:8080/devices/<SERIAL>/warranty \
  | python3 -m json.tool
# confirms OEM warranty status and KEBS cert validity
```

4. If in-warranty, proceed to OEM claim (Phase 3).
5. If out-of-warranty, escalate to procurement for replacement purchase.

### Phase 3: OEM claim (Field Support -> OEM/Vendor)

```bash
# record RMA initiation:
curl -sf -X POST http://inventory-svc.eel-system.svc.cluster.local:8080/devices/<SERIAL>/events \
  -H "Content-Type: application/json" \
  -d "{\"event\":\"device.rma\",\"rma_ticket\":\"<OEM_RMA_NUMBER>\",\"officer\":\"<OFFICER_ID>\"}"
```

6. Contact OEM using `coverage-matrix.yaml` for vendor contact details.
7. Obtain RMA number from OEM.
8. Wipe device before shipping (see MDM wipe procedure in `operations/mdm/lifecycle.md`).
9. Ship device to OEM.

### Phase 4: return and re-provisioning (eel Inventory -> Field Support)

```bash
# record replacement device receipt:
curl -sf -X POST http://inventory-svc.eel-system.svc.cluster.local:8080/devices \
  -H "Content-Type: application/json" \
  -d "{\"serial\":\"<NEW_SERIAL>\",\"imei\":\"<NEW_IMEI>\",\"model\":\"<MODEL>\",\"category\":\"<CATEGORY>\",\"replaces_serial\":\"<OLD_SERIAL>\"}"
```

10. Receive replacement device from OEM.
11. Verify KEBS conformity cert for replacement device.
12. Provision and enrol replacement via MDM (see `operations/mdm/lifecycle.md`).
13. Close Zammad ticket.
14. Update Zammad ticket and notify field staff.

## RMA tracking

```bash
# list all open RMAs:
curl -sf "http://inventory-svc.eel-system.svc.cluster.local:8080/devices/rma?status=open" \
  | python3 -m json.tool

# get RMA history for a device:
curl -sf "http://inventory-svc.eel-system.svc.cluster.local:8080/devices/<SERIAL>/events?type=device.rma" \
  | python3 -m json.tool
```
