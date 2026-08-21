# Kit Lifecycle

This guide covers the full lifecycle of a field kit device from procurement through decommissioning, referencing the inventory-svc (M20) and biometric-edge-svc (M01/M05/M24) modules and the MDM system.

## 1. Procurement and Receiving

Kit hardware (Android devices with biometric sensors) is received from approved OEM vendors. On arrival:

1. Verify KEBS certification and OEM warranty document.
2. Scan device serial barcode into `inventory-svc` via `just kit-receive <serial>`.
3. A `CustodyEvent(type=RECEIVED)` is recorded with the receiving officer identity.
4. Device enters `RECEIVED` status in inventory-svc.

## 2. MDM Enrolment

```sh
just mdm-enroll <serial>
```

This triggers Android Enterprise zero-touch enrolment. The MDM server (at `operations/mdm/`) pushes the base profile and biometric-edge firmware. The device reboots into managed mode and registers with inventory-svc as `MDM_ENROLLED`.

## 3. Kit Provisioning (inventory-svc: M20)

Before deployment:

1. Assign the kit to a polling station: `just kit-assign <serial> <station-id>`.
2. inventory-svc emits `eel.inventory.kit.assigned`.
3. biometric-edge-svc picks up the assignment and downloads the register extract and kit config bundle from bvr-svc and bvi-svc.
4. Kit status changes to `PROVISIONED`.

## 4. Pre-deployment Verification

Run the kit self-test procedure documented in `operations/mdm/pre-deploy-checklist.md`:
- Biometric sensor ISO certification check
- GNSS lock verification
- Network connectivity to nearest edge cluster
- SPIFFE SVID validity check
- Kit certificate issuance confirmation (60-second revocation path verified)

## 5. Election-Day Operation

During polling, kits operate with bvi-svc in online mode (M05) or offline mode if connectivity is lost. All events queue on-device and flush on reconnect. Daily kit health is reported to ECC (M16) via telemetry.

## 6. End-of-Day and Collection

After polling closes, the kit transmits its final Form 34A batch (M08). The collection officer scans the kit seal with the inventory app (M20), recording `CustodyEvent(type=COLLECTED)`. The device returns to `COLLECTED` status.

## 7. Post-election Audit

For kits selected in RLA sampling (M21), device forensic logs are exported before wipe. The governance-svc evidence bundle procedure in `docs/runbooks/` covers the chain of custody for audited kits.

## 8. Decommissioning

When a kit reaches end-of-life:

1. `just mdm-wipe <serial>` - triggers factory reset; biometric data deleted on-device.
2. inventory-svc records `CustodyEvent(type=DECOMMISSIONED)`.
3. Kit certificate is revoked in the bvi-svc CRL.
4. Physical device is disposed of per WEEE regulations (NEMA guidance in `operations/warranty/`).

## Related

- `inventory-svc` architecture: [docs/architecture/services/inventory.md](../architecture/services/inventory.md)
- `biometric-edge-svc` architecture: [docs/architecture/services/biometric-edge.md](../architecture/services/biometric-edge.md)
- MDM operations: `operations/mdm/`
- Warranty procedures: `operations/warranty/`
