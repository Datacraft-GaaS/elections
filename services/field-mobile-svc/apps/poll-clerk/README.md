# eel poll-clerk (M31)

Poll clerk companion app for election day. Runs on MDM-provisioned Android tablets at polling stations. Records incidents and irregularities, manages end-of-day (EOD) report submission, and provides adjacency checks with the BVI kit.

## Features

- Incident recording with timestamp, location, and photo evidence
- End-of-day (EOD) report generation and submission
- Form 34A validation and signing
- Adjacency checks with BVI kit device (Bluetooth/local network)
- Event encryption and local queue for offline capability
- Play Integrity attestation
- 11-language UI
- WCAG 2.2 AA accessibility
- Companion mode for BVI kit synchronization

## Build

```bash
cd services/field-mobile-svc/apps/poll-clerk
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
flutter test
flutter test integration_test/
flutter test --machine test/ | patrol
```

## Environment

Configure via `assets/config/`:
- `API_BASE_URL`: field-mobile-svc backend endpoint
- `POLLING_STATION_ID`: station identifier
- `BVI_KIT_COMPANION_PORT`: local port for BVI kit communication

## Device Requirements

- Android SDK 33+
- 2GB RAM minimum
- Camera (for incident photos)
- MDM enrollment via Jamf Mobile (policy: eel-poll-clerk-mdm.json)

## Release

Internal Google Play track (eel_poll_clerk).

## Architecture

- **Frontend**: Flutter + Dart
- **Backend**: gRPC client to field-mobile-svc/crates/mobile-bff
- **Storage**: SQLite encrypted queue for incidents
- **Attestation**: Android Play Integrity

## Data Flow

1. Incident recording: capture description, photo, timestamp
2. Local encryption: sign and queue incident
3. BVI kit adjacency: periodic check for kit presence via local network
4. EOD report: compile incidents and Form 34A, submit via field-mobile-svc
5. Sync: transmit queued events on network reconnect

## EOD Report

The EOD report includes:
- All recorded incidents with evidence
- Form 34A tally summary
- Device attestation and signature
- Polling station closure timestamp

## Connectivity

- Primary: cellular data (Safaricom/Airtel)
- Fallback: local queue with retry on reconnect
- Adjacency: Bluetooth Low Energy (BLE) with BVI kit

## Related Services

- **field-mobile-svc**: mobile BFF backend (mobile-bff crate)
- **bvi-svc**: BVI kit event storage
