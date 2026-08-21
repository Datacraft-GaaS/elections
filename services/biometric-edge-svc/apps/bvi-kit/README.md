# eel bvi-kit (M05)

Biometric Voter Identification (BVI) kit app for election day. Runs on hardened MDM-provisioned Android tablets at polling stations. Identifies voters by fingerprint and iris against on-device template extracts, records identification events, manages polling session lifecycle (open/close), and generates end-of-day reports.

## Features

- Fingerprint and iris identification against local template database
- Real-time voter identification with sub-second latency
- Polling session management (open, close, reopen)
- Event encryption and signing for non-repudiation
- 7-day encrypted offline event queue (AES-256-GCM, SQLite)
- Automatic sync on reconnect
- Play Integrity attestation
- 11-language UI
- WCAG 2.2 AA accessibility
- End-of-day report generation with signed summary

## Build

```bash
cd services/biometric-edge-svc/apps/bvi-kit
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
flutter test integration_test/            # integration tests
flutter test --machine test/ | patrol     # e2e tests
```

## Environment

Configure via `assets/config/`:
- `API_BASE_URL`: BVI event ingest endpoint
- `KIT_SESSION_ID`: polling station session identifier
- `REGISTER_EXTRACT_PATH`: path to pre-loaded biometric templates

## Device Requirements

- Android SDK 33+
- 2GB RAM minimum
- Biometric sensor suite (fingerprint + iris)
- MDM enrollment via Jamf Mobile (policy: eel-bvi-kit-mdm.json)

## Release

Internal Google Play track (eel_bvi_kit).

## Architecture

- **Frontend**: Flutter + Dart, sensor integration via platform channels
- **Backend**: gRPC client connecting to biometric-edge-svc/crates/edge-api
- **Storage**: SQLite encrypted queue for events
- **Attestation**: Android Play Integrity + kit-attest NIF

## Data Flow

1. Session open: create session record, load templates, initialize kit
2. Voter identification: capture fingerprint/iris, match against templates
3. Event recording: sign identification event, enqueue
4. Sync: transmit queued events to bvi-svc on network reconnect
5. Session close: generate EOD report with signed summary

## Performance SLA

- Identification scan + sign + transmit: <= 90 seconds
- Template matching: <= 5 seconds per capture
- Offline queue drain: <= 2 hours

## Related Services

- **biometric-edge-svc**: edge-api gRPC service
- **bvi-svc**: event ingest and persistence
