# custody-mobile

Flutter custody chain app for inventory officers to track election asset movements via RFID and QR.

## Module

M20 (inventory-management) | inventory-svc | WP-4.3

## Purpose

MDM-provisioned Android app for inventory officers at warehouses and county stores.
Scans RFID tags and QR codes on election assets (kits, ballot boxes, ballot paper
lots, seals, consumables), records custody transfer events (store-in, store-out, seal
application, seal break), and submits to inventory-svc. Designed for use in warehouse
environments with patchy connectivity.

## Contents

```
custody-mobile/
  lib/              Dart/Flutter source
  test/             Unit tests
  integration_test/ Integration tests
  assets/           Config, icons
  pubspec.yaml
```

## Target Platform

Android (SDK 33+). MDM-provisioned (Jamf Mobile, policy: eel-custody-mdm.json).

## Offline Capabilities

7-day local custody event queue (SQLite, AES-256-GCM). Events transmitted on reconnect.
RFID reader connected via USB-OTG or Bluetooth.

## Attestation Profile

Android Play Integrity attestation.

## i18n / WCAG

- Languages: English + Swahili
- WCAG: 2.2 AA

## Build Commands

```bash
cd services/inventory-svc/apps/custody-mobile
flutter build apk --release --flavor production
flutter build apk --release --flavor staging
```

## Configuration / Env Variables

Set via `assets/config/env.json`:

| Key | Description |
|---|---|
| `API_BASE_URL` | inventory-svc API |
| `OFFICER_BADGE_ID` | Provisioned officer identifier |
| `RFID_READER_URL` | RFID reader connection (USB-OTG or BT) |
| `KEBS_ADAPTER_URL` | KEBS warranty registration adapter |

## Test Commands

```bash
flutter test
flutter test integration_test/
```

## Deployment Notes

- Google Play internal track (ke.go.iebc.eel.custody_mobile)
- MDM policy: platform/mdm/eel-custody-mdm.json
- Provisioned alongside kit devices at county ICT hubs

## Store Submission Checklist

- [ ] Increment versionCode
- [ ] Sign with production keystore
- [ ] Upload to Play Console internal track
- [ ] Verify on target warehouse device hardware

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M20)
- integrations/kebs-warranty
- docs/runbooks/inventory-custody-chain.md

## Upstream / Downstream

- Upstream: inventory-svc/crates/custody, integrations/kebs-warranty
- Downstream: inventory-svc (event ingest), integrations/printer-mes
