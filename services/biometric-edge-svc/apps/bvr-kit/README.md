# eel bvr-kit (M01)

Biometric Voter Registration (BVR) kit app for Android tablets. Runs on hardened MDM-provisioned devices during voter registration drives. Captures biometric data (fingerprints, iris scans), passport/ID MRZ, and photos. Digitally signs enrolment events and queues them for transmission to the backend.

## Features

- Fingerprint and iris biometric capture via calibrated sensors
- MRZ (Machine Readable Zone) scanning for passport and ID validation
- Photo capture with on-device validation
- Cryptographic signing of enrolment events
- 7-day encrypted offline queue (AES-256-GCM, SQLite)
- Automatic sync on network reconnect
- SafetyNet/Play Integrity attestation
- 11-language UI (Swahili, English, Kikuyu, Luo, Luhya, Kalenjin, Kamba, Kisii, Meru, Somali, Maasai)
- WCAG 2.2 AA accessibility

## Build

```bash
cd services/biometric-edge-svc/apps/bvr-kit
flutter build apk --release --flavor production
flutter build apk --release --flavor staging
```

## Development

```bash
flutter pub get
flutter run --flavor development
```

## Tests

```bash
flutter test                              # unit tests
flutter test integration_test/            # integration tests (requires device/emulator)
flutter test --machine test/ | patrol     # e2e tests with patrol
```

## Environment

Configure via `assets/config/`:
- `ENV_NAME`: production, staging, or development
- `API_BASE_URL`: enrolment backend gRPC endpoint
- `KIT_ATTEST_URL`: kit attestation verification endpoint
- `MRZ_SDK_LIB_PATH`: path to MRZ recognition library

## Device Requirements

- Android SDK 33+
- 2GB RAM minimum
- Biometric sensor suite (fingerprint + iris)
- MDM enrollment via Jamf Mobile (policy: eel-bvr-kit-mdm.json)

## Release

Internal Google Play track (eel_bvr_kit). Version code managed by CI pipeline.

## Architecture

- **Frontend**: Flutter + Dart, sensor integration via platform channels
- **Backend**: gRPC client connecting to biometric-edge-svc/crates/edge-api
- **Storage**: SQLite encrypted queue, AES-256-GCM for at-rest data
- **Attestation**: Android Play Integrity + custom kit-attest Rust NIF

## Documentation

- Threat model: docs/THREAT-MODEL.md (kit section)
- MDM policy: platform/mdm/eel-bvr-kit-mdm.json
- Sensor integration guide: docs/sensor-integration.md

## Performance SLA

- Registration cycle: <= 5 minutes per voter
- Biometric capture: <= 90 seconds
- Offline queue drain: <= 2 hours (post-reconnect)
