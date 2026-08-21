# Android Enterprise remote command catalog

These commands are issued from the MDM console or via the MDM API. All commands are logged to the MDM audit trail and, for kit devices, emit a leaf to `transparency-svc`.

## Available commands

| Command | API action | Effect | Requires dual-officer |
|---------|-----------|--------|----------------------|
| Lock device | `LOCK` | Immediately locks the screen and requires PIN to unlock | No |
| Wipe device | `RESET_PASSWORD` then `WIPE` | Factory reset; all data erased | Yes |
| Reboot | `REBOOT` | Restarts the device | No |
| Push config update | `APPLY_POLICY` | Applies latest MDM policy without reboot | No |
| Enable honey mode | `SET_CUSTOM_PARAMETER honey_mode=true` | Activates evidence capture (see stolen-kit-honey-mode.md) | Yes |
| Revoke enrolment | `UNENROLL` | Removes MDM management; triggers kit-mass-revocation.md | Yes |
| Collect logs | `COLLECT_DIAGNOSTICS` | Uploads device logs to MDM console | No |
| Enable lost mode | `ENABLE_LOST_MODE` | Displays contact message on screen; location reported every 15 min | No |
| Disable lost mode | `DISABLE_LOST_MODE` | Returns device to normal operation | No |

## Usage

All commands can be issued from the MDM console web UI or via the MDM REST API:

```bash
# example: lock a specific kit
curl -sf -X POST https://mdm.eel.io/api/v1/devices/<DEVICE_ID>/commands \
  -H "Authorization: Bearer <MDM_API_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{"command":"LOCK","ticket":"<TICKET>"}'

# example: push policy update to all kits in a group
curl -sf -X POST https://mdm.eel.io/api/v1/groups/eel-kits/commands \
  -H "Authorization: Bearer <MDM_API_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{"command":"APPLY_POLICY","policy_version":"<VERSION>","ticket":"<TICKET>"}'
```

## Audit

Every remote command is logged to the MDM audit trail with the issuing officer ID, timestamp, and ticket reference. For kit wipes and honey-mode activations, a tlog leaf is emitted automatically by the MDM webhook integration.
