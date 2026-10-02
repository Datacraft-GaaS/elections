# accred-scanner

QR/barcode scanner Flutter app for verifying media and observer accreditation at entry points.

## Module

M42 (media-accreditation), M43 (observer-accreditation) | training-svc | WP-5.4

## Purpose

MDM-provisioned Android app deployed at polling centre and tallying centre entry gates.
Security personnel scan accreditation QR codes on press badges and observer credentials.
The app verifies the cryptographic signature on the credential bundle offline (using a
locally stored CRL and trusted root), displays the holder's name and accreditation tier,
and records the visit event. Network connectivity is not required for verification.

## Contents

```
accred-scanner/
  lib/                Dart/Flutter source
  test/               Unit tests
  integration_test/   Integration tests
  assets/             CRL bundle, trusted root cert, config
  pubspec.yaml
```

## Target Platform

Android (SDK 33+). MDM-provisioned (Jamf Mobile, policy: eel-accred-scanner-mdm.json).

## Offline Capabilities

Full offline verification using locally stored signed credential bundle and CRL.
CRL bundle refreshed daily via MDM push (or manual sync). Visit events buffered locally
and synced to training-svc when online.

## Attestation Profile

Android Play Integrity attestation.

## i18n / WCAG

- Language: English (gate personnel use)
- WCAG: 2.2 AA

## Build Commands

```bash
cd services/training-svc/apps/accred-scanner
flutter build apk --release --flavor production
flutter build apk --release --flavor staging
```

## Configuration / Env Variables

| Key | Description |
|---|---|
| `API_BASE_URL` | training-svc API |
| `SCANNER_STATION_ID` | Provisioned entry point identifier |
| `CRL_BUNDLE_PATH` | Local path to signed CRL bundle |
| `TRUSTED_ROOT_CERT` | Trusted root for credential signature verification |

## Test Commands

```bash
flutter test
flutter test integration_test/
```

## Deployment Notes

- Google Play internal track (ke.go.commission.eel.accred_scanner)
- MDM policy: platform/mdm/eel-accred-scanner-mdm.json
- CRL bundle updated via MDM before each major event (nomination day, election day)

## Store Submission Checklist

- [ ] Increment versionCode
- [ ] Sign with production keystore
- [ ] Upload to Play Console internal track
- [ ] Test offline scan of revoked and valid credentials

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M42, M43)
- training-svc/django/accreditation
- integrations/mck (MCK accreditation upstream)
- docs/runbooks/accreditation-gate-ops.md

## Upstream / Downstream

- Upstream: training-svc/crates/training-api (credential bundles + CRL)
- Downstream: training-svc (visit event records)
