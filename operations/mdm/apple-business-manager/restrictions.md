# Apple Business Manager (ABM) restrictions: iOS staff devices

## Scope

iOS devices issued to election commission staff via Apple Business Manager. Devices are supervised and DEP-enrolled; profile removal is disallowed without MDM authorisation.

## Restrictions applied

| Restriction | Setting | Rationale |
|-------------|---------|-----------|
| Screenshot | Disabled | Sensitive electoral data on screen |
| Camera | Enabled | Staff require camera for document scanning |
| AirDrop | Disabled | Data exfiltration vector |
| iCloud backup | Disabled | Organisational data must not leave the MDM-controlled environment |
| Managed app iCloud sync | Disabled | Prevents data leakage via iCloud |
| USB drive file access | Disabled | Prevents uncontrolled data transfer |
| Network drive file access | Disabled | Same rationale |
| Encrypted backup | Enforced | All local backups must be encrypted |
| Untrusted TLS prompt | Disabled | Prevents MITM via custom CA acceptance |
| Wi-Fi allowlist | Enforced | Only approved SSIDs allowed |
| Passcode | Alphanumeric, 8 chars minimum | Stronger than default numeric PIN |
| Passcode max age | 90 days | Periodic rotation |
| Max inactivity | 5 minutes | Reduces exposure for unattended devices |
| Profile removal | Disallowed | Cannot unenroll without MDM authorisation |

## Enforcement

Devices that fail compliance checks (e.g., passcode not set, OS outdated) are quarantined automatically. Staff must contact IT support to resolve compliance failures before regaining access to eel apps.
