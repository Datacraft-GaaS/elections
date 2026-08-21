# eel field-support (M32)

Field support technician app for election day operations. Enables remote diagnostics, device management, and support ticket creation. Used by field engineers to troubleshoot kit devices and coordinate repairs.

## Features

- Remote device diagnostics (sensor health, connectivity, battery)
- Support ticket creation with automatic device telemetry
- Device status dashboard (real-time poll of kit devices)
- Remote action whitelisting (controlled remote troubleshooting)
- Offline ticket queue (for offline areas)
- Play Integrity attestation (elevated permissions)
- 2-language UI (English, Swahili)
- WCAG 2.2 AA accessibility

## Build

```bash
cd services/field-mobile-svc/apps/field-support
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
```

## Environment

Configure via `assets/config/`:
- `API_BASE_URL`: field-mobile-svc backend endpoint
- `TECH_BADGE_ID`: technician identifier
- `REMOTE_ACTION_WHITELIST`: comma-separated list of allowed remote actions

## Device Requirements

- Android SDK 33+
- 1.5GB RAM minimum
- MDM enrollment via Jamf Mobile (elevated permissions policy)

## Release

Internal Google Play track (eel_field_support).

## Diagnostics

Remote diagnostics include:
- Biometric sensor status and calibration
- Battery health and charge cycles
- Network connectivity (cellular, WiFi, offline queue status)
- GPS accuracy and last known location
- Disk space and queue depth
- SafetyNet/Play Integrity status

## Support Tickets

Each ticket captures:
- Device ID and location
- Issue category (sensor failure, connectivity, software crash)
- Technician notes and photos
- Automatic telemetry snapshot
- Priority level
- Escalation path

Tickets transmitted via field-mobile-svc to support-admin-svc for assignment and tracking.

## Remote Actions

Whitelisted remote actions only:
- Device restart
- Sensor recalibration request
- Cache clear
- Log dump download
- Connectivity test

No destructive actions (factory reset, uninstall app).

## Architecture

- **Frontend**: Flutter + Dart
- **Backend**: gRPC client to field-mobile-svc/crates/support-api
- **Storage**: SQLite encrypted queue for tickets
- **Attestation**: Android Play Integrity (elevated)

## Related Services

- **field-mobile-svc**: support API backend (support-api crate)
- **inventory-svc**: device management and kit status
- **support-admin-svc**: ticket assignment and tracking
