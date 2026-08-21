# Android Enterprise restrictions

## Kit devices (BVI, BVR, RTS)

| Restriction | Setting | Rationale |
|-------------|---------|-----------|
| Unknown sources | Disabled | Prevents sideloading of unauthorised apps |
| Google Play Protect | Enforced | Malware scanning on all installed apps |
| Hardware attestation | Required | Cryptographic device integrity proof at enrolment and check-in |
| Screenshot | Disabled | Prevents capture of sensitive voter or tally data |
| Factory reset | Disabled | Prevents wiping without MDM authorisation |
| Wi-Fi configuration | Disabled | Prevents connecting to unauthorised networks |
| App allowlist | eel apps only | No general-purpose apps; kiosk launcher enforced |
| Screen timeout | 5 minutes | Reduces exposure if kit is unattended |
| Encryption | Enabled with PIN | Data at rest is encrypted |
| Location | High accuracy | Required for GPS check-in anti-spoofing |
| System navigation | Disabled | Kiosk mode; operator cannot leave the eel app |
| Status bar | Disabled | Prevents access to quick settings in kiosk mode |

## Mobile devices (party agent, observer, support)

| Restriction | Setting | Rationale |
|-------------|---------|-----------|
| Unknown sources | Disabled | Prevents sideloading |
| Google Play Protect | Enforced | Malware scanning |
| Hardware attestation | Required | Device integrity |
| Screenshot | Disabled | Sensitive data protection |
| App allowlist | eel field apps only | Limits attack surface |
| Screen timeout | 10 minutes | Longer than kit devices; operators may use devices intermittently |
| Encryption | Enabled with PIN | Data at rest protected |
| Location | High accuracy | Anti-spoofing for field check-ins |

## Enforcement

Devices that do not meet the compliance policy are automatically quarantined by the MDM: network access is restricted to the MDM management endpoint only, and the device operator receives a notification to contact field support.
