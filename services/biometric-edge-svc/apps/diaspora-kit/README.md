# eel diaspora-kit (M24)

Diaspora voting kit app for Android and iOS. Deployed at embassy polling stations for diaspora voter identification and Form 34AD record submission. Handles satellite and VPN fallback connectivity and manages ballot custody chain events.

## Features

- Fingerprint and iris identification for diaspora voters
- Form 34AD digitized record capture and submission
- Ballot pouch custody event tracking (seal application/break, custody transfer)
- Satellite and VPN fallback for embassy connectivity
- Full offline capability with encrypted local queue
- iOS DeviceCheck + Android Play Integrity attestation
- 2-language UI (English, Swahili)
- WCAG 2.2 AA accessibility
- Integration with KRA e-clearance for ballot containers
- Airline cargo tracking integration

## Build

```bash
# Android
flutter build apk --release --flavor diaspora

# iOS (requires macOS + Xcode)
flutter build ios --release --flavor diaspora
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
```

## Environment

Configure via `assets/config/`:
- `API_BASE_URL`: diaspora voting backend endpoint
- `EMBASSY_STATION_ID`: embassy station identifier
- `VPN_FALLBACK_ENDPOINT`: VPN gateway for connectivity fallback
- `SATELLITE_ENABLED`: enable satellite modem (boolean)

## Device Requirements

### Android
- SDK 33+
- 2GB RAM minimum
- Biometric sensor suite

### iOS
- iOS 13+
- 2GB RAM minimum
- Biometric sensor suite (if available)

## Release

- **Android**: internal Google Play internal track (eel_diaspora_kit)
- **iOS**: Apple Enterprise certificate, internal App Store distribution

## Architecture

- **Frontend**: Flutter + Dart, cross-platform biometric integration
- **Backend**: gRPC client to biometric-edge-svc/crates/edge-api
- **Storage**: SQLite encrypted queue (AES-256-GCM)
- **Connectivity**: primary VPN, fallback satellite modem

## Data Flow

1. Voter identification: fingerprint/iris against embassy register
2. Form 34AD: digitize and sign official record
3. Custody events: track ballot pouch seals and transfers
4. Transmission: queue events, sync when connectivity available

## Integrations

- **KRA e-clearance**: ballot container pre-clearance (integrations/kra-eclearance)
- **Airline cargo**: tracking integration (integrations/airline-cargo)
- **Satellite modem**: Iridium or equivalent for remote embassies

## Performance SLA

- Identification: <= 90 seconds
- Offline queue drain: <= 2 hours post-reconnect
