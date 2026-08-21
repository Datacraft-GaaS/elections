# operations/mdm

Mobile Device Management for eel. Manages the full fleet of KIEMS election devices and
Commission staff devices across two MDM platforms: Android Enterprise for field kits and
mobile apps, Apple Business Manager for iOS staff devices.

## Overview

eel uses two MDM platforms:

- Android Enterprise (AE): manages BVI kits, BVR kits, RTS kits, and Android apps for
  party agents, observers, and field support staff. Profiles in `android-enterprise/`.
- Apple Business Manager (ABM): manages iOS devices issued to Commission staff.
  Profiles in `apple-business-manager/`.

All devices are enrolled under the eel organisation account. No personal devices are enrolled.
BYOD is not permitted for any role that accesses election data.

## Device categories

| Category | Platform | Profile | Users |
|----------|----------|---------|-------|
| BVI kit | Android Enterprise | `profile-kit.json` | Identification officers |
| BVR kit | Android Enterprise | `profile-kit.json` | Registration officers |
| RTS kit | Android Enterprise | `profile-kit.json` | Results transmission officers |
| Party agent mobile | Android Enterprise | `profile-mobile.json` | Accredited party agents |
| Observer mobile | Android Enterprise | `profile-mobile.json` | Accredited observers |
| Field support mobile | Android Enterprise | `profile-mobile.json` | Field support staff |
| Staff iOS | Apple Business Manager | `profile-staff.mobileconfig` | Commission staff |

## Key policies (all Android Enterprise devices)

- System update floor: latest quarterly security patch required; devices with outdated
  patches are quarantined automatically.
- No sideloading: unknown sources disabled; only allowlisted Play Store apps permitted.
- Attestation: Android hardware attestation enforced at enrolment and on every check-in.
- Screenshots disabled on eel apps.
- Remote wipe capability: full device wipe via MDM console in case of loss or compromise.

## Device lifecycle

Full lifecycle documented in `lifecycle.md`:

1. Provision: enrol device, apply profile, install eel apps, run attestation check.
2. Deploy: ship to polling station or assign to officer.
3. Active: device checks in every 30 seconds during election day; MDM monitors connectivity.
4. Replace: on hardware failure, trigger RMA via `operations/warranty/rma-workflow.md`.
5. Wipe and decommission: after the petition window, remote wipe and return to inventory.

## Usage

```bash
# Validate an Android Enterprise profile against AE JSON schema
python3 -m jsonschema \
  -i android-enterprise/profile-kit.json \
  android-enterprise/ae-profile.schema.json

# Export enrolled device list (requires MDM console API credentials)
curl -s -H "Authorization: Bearer $MDM_TOKEN" \
  https://mdm.eel.internal/api/v1/devices \
  | jq '.[].serialNumber'
```

## Files

| File | Description |
|------|-------------|
| `android-enterprise/profile-kit.json` | AE declarative profile for BVI/BVR/RTS kits |
| `android-enterprise/profile-mobile.json` | AE profile for party agent, observer, and field support |
| `android-enterprise/restrictions.md` | Restriction policy explanation and rationale |
| `android-enterprise/remote-command-catalog.md` | Remote commands available via MDM console |
| `apple-business-manager/profile-staff.mobileconfig` | ABM profile for iOS staff devices |
| `apple-business-manager/restrictions.md` | iOS restriction policy explanation |
| `lifecycle.md` | Full device lifecycle: provision, deploy, replace, wipe, decommission |

## Related milestones

- M55 (HR integration): `lifecycle.md` ties device lifecycle to HR events.
- M32 (field support): device replacement and RMA flows reference the remote command catalog.
- M20 (inventory): device custody events emitted to `inventory-svc` at every lifecycle stage.

## Dependencies

- Android Enterprise MDM platform (Google API integration)
- Apple Business Manager (Apple API integration)
- `inventory-svc`: receives device custody events via outbox events
- `field-mobile-svc` (M32): triggers MDM remote commands on kit failure reports

## Limitations

- MDM platform credentials are not stored in this repo; retrieve from vault before running
  MDM API commands.
- Profile changes take effect on the next device check-in (up to 30 seconds on election day).
  Emergency profile pushes can use the remote command catalog.
- AE profiles are per-device-category; a kit enrolled under the wrong profile must be
  unenrolled and re-enrolled with the correct profile.
