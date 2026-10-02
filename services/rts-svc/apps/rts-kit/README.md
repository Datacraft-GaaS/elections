# rts-kit

Results Transmission System kit app for polling station Form 34A scanning and transmission.

## Module

M08 (rts-device) | rts-svc | WP-3.1

## Purpose

Flutter Android app running on MDM-provisioned kit devices at each polling station.
Poll clerks use it to scan Form 34A after tallying. The app compresses and signs the
scan, attaches a participation token, and transmits to rts-svc via the primary VPN
link. Falls back to SMS-compressed payload when VPN is unavailable. Maintains a 7-day
offline queue.

## Contents

```
rts-kit/
  lib/                Dart/Flutter source
  test/               Unit tests
  integration_test/   Integration tests (requires device)
  assets/             Fonts, icons, form overlays
  pubspec.yaml        Flutter dependencies
```

## Target Platform

Android (SDK 33+). MDM-provisioned kit device (Jamf Mobile, policy: eel-rts-kit-mdm.json).

## Offline Capabilities

7-day local event queue (SQLite, AES-256-GCM encrypted). Transmission auto-retried on
reconnect. SMS fallback compresses signed scan to within SMS payload limits using brotli
+ base85.

## Attestation Profile

Android Play Integrity attestation + kit-attest Rust NIF (via biometric-edge-svc/crates/kit-attest).

## i18n / WCAG

- Languages: English + Swahili (Returning Officer use)
- WCAG: 2.2 AA

## Build Commands

```bash
cd services/rts-svc/apps/rts-kit

# Production APK
flutter build apk --release --flavor production

# Staging APK
flutter build apk --release --flavor staging

# Split ABIs (for Play internal track)
flutter build apk --release --split-per-abi --flavor production
```

## Configuration / Env Variables

Set via `assets/config/env.json` (injected at build by CI):

| Key | Description |
|---|---|
| `API_BASE_URL` | rts-svc device API endpoint |
| `RTS_STATION_ID` | Unique polling station identifier |
| `SMS_SHORTCODE` | Telco SMS shortcode for fallback transmission |
| `PARTICIPATION_TOKEN_ENDPOINT` | Token fetch endpoint |
| `KIT_ATTEST_URL` | kit-attest service endpoint |

## Test Commands

```bash
flutter test                            # unit tests
flutter test integration_test/          # integration (requires connected device/emulator)
flutter test --machine test/ | patrol   # patrol e2e
```

## Deployment Notes

- Distributed via Google Play internal track (package: ke.go.commission.eel.rts_kit)
- MDM policy in platform/mdm/eel-rts-kit-mdm.json
- Device provisioning: Jamf Mobile device enrollment before election day

## SLA

- Scan + sign + transmit: p95 target <= 90s (M08 SLA)

## Store Submission Checklist

- [ ] Increment `versionCode` in `pubspec.yaml`
- [ ] Generate signed APK with production keystore
- [ ] Upload to Google Play Console (internal track)
- [ ] Verify Play Integrity attestation passes on target device model
- [ ] Confirm MDM policy pushes update before election day

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M08)
- integrations/telco-sms (SMS fallback)
- biometric-edge-svc/crates/kit-attest
- docs/runbooks/rts-kit-provisioning.md

## Upstream / Downstream

- Upstream: rts-svc/crates/rts-device-api, integrations/telco-sms
- Downstream: rts-svc (event ingest)
