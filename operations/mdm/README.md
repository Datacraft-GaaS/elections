# MDM: Mobile Device Management

## Overview

eel uses two MDM platforms:

- **Android Enterprise (AE):** manages BVI kits, BVR kits, RTS kits, and the Android apps for party agents, observers, and support mobile staff. Profiles are in `android-enterprise/`.
- **Apple Business Manager (ABM):** manages iOS devices issued to election commission staff. Profiles are in `apple-business-manager/`.

All devices are enrolled under the eel organisation account. No personal devices are enrolled. BYOD is not permitted for any role that accesses election data.

## Device categories

| Category | Platform | Profile | Users |
|----------|----------|---------|-------|
| BVI kit | Android Enterprise | profile-kit.json | Identification officers |
| BVR kit | Android Enterprise | profile-kit.json | Registration officers |
| RTS kit | Android Enterprise | profile-kit.json | Results transmission officers |
| Party agent mobile | Android Enterprise | profile-mobile.json | Accredited party agents |
| Observer mobile | Android Enterprise | profile-mobile.json | Accredited observers |
| Support mobile | Android Enterprise | profile-mobile.json | Field support staff |
| Staff iOS | Apple Business Manager | profile-staff.mobileconfig | Commission staff |

## Key policies (all Android Enterprise devices)

- System update floor: latest quarterly security patch required; devices with outdated patches are quarantined.
- No sideloading: unknown sources disabled; only allowlisted Play Store apps permitted.
- Attestation required: Android hardware attestation enforced at enrolment and on every check-in.
- No screenshots on sensitive apps: screenshot disabled for eel apps.

## Related milestones

- M55 (HR integration): `operations/mdm/lifecycle.md` ties device lifecycle to HR events.
- M32 (field support): device replacement and RMA flows reference `operations/mdm/android-enterprise/remote-command-catalog.md`.
- M20 (inventory): device custody events emitted to `inventory-svc` at every lifecycle stage.

## Files

| File | Description |
|------|-------------|
| `android-enterprise/profile-kit.json` | AE declarative profile for BVI/BVR/RTS kits |
| `android-enterprise/profile-mobile.json` | AE declarative profile for party agent/observer/support mobile |
| `android-enterprise/restrictions.md` | Restriction policy explanation and rationale |
| `android-enterprise/remote-command-catalog.md` | Remote commands available via MDM console |
| `apple-business-manager/profile-staff.mobileconfig` | ABM profile for iOS staff devices |
| `apple-business-manager/restrictions.md` | iOS restriction policy explanation |
| `lifecycle.md` | Full device lifecycle: provision, wipe, decommission, RMA |
