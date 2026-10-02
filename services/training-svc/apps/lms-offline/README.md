# lms-offline

Offline-capable LMS Flutter app for poll worker training certification.

## Module

M38 (poll-worker-training-lms) | training-svc | WP-5.4

## Purpose

MDM-provisioned Android app for poll workers to complete mandatory training modules
offline. Trainees download course content on initial sync, complete quizzes and
assessments locally, and receive a digitally signed certificate when thresholds are
met. Certificates include a QR code for on-site verification by the accred-scanner app.
Syncs completions and certificates when connectivity is available.

## Contents

```
lms-offline/
  lib/                Dart/Flutter source
  test/               Unit tests
  integration_test/   Integration tests
  assets/             Course metadata placeholders, config
  pubspec.yaml
```

## Target Platform

Android (SDK 33+). MDM-provisioned (Jamf Mobile, policy: eel-lms-mdm.json).

## Offline Capabilities

Full offline: course content downloaded on first sync (video + text modules). Assessment
results buffered locally. Certificate generation triggers on sync-back to training-svc.

## Attestation Profile

Android Play Integrity attestation.

## i18n / WCAG

- Languages: 11 Kenyan languages (Swahili, English, Kikuyu, Luo, Luhya, Kalenjin, Kamba, Kisii, Meru, Somali, Maasai)
- WCAG: 2.2 AA

## Build Commands

```bash
cd services/training-svc/apps/lms-offline
flutter build apk --release --flavor production
flutter build apk --release --flavor staging
```

## Configuration / Env Variables

| Key | Description |
|---|---|
| `API_BASE_URL` | training-svc API |
| `LMS_COURSE_SYNC_URL` | Course bundle download endpoint |
| `CERT_SIGN_KEY` | Public key for certificate verification |
| `WORKER_ID` | Provisioned poll worker identifier |

## Test Commands

```bash
flutter test
flutter test integration_test/
```

## Deployment Notes

- Google Play internal track (ke.go.commission.eel.lms_offline)
- MDM policy: platform/mdm/eel-lms-mdm.json
- Course bundles pre-downloaded at county training sessions

## Store Submission Checklist

- [ ] Increment versionCode
- [ ] Sign with production keystore
- [ ] Upload to Play Console internal track
- [ ] Test full offline course completion flow on target device

## Related Docs

- docs/plans/12-appendix-module-inventory.md (M38)
- training-svc/django/lms
- training-svc/crates/training-api
- training-svc/apps/accred-scanner (certificate verification)

## Upstream / Downstream

- Upstream: training-svc/crates/training-api (course bundles + cert signing)
- Downstream: training-svc/django/lms (enrollment records)
