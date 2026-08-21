# eel legacy-android (M01/M05 shim)

Replacement app for ~14,100 legacy Huawei and Samsung field devices still in inventory. Implements gRPC shim protocol to match the legacy-android-shim integration. Targets older ARM32 devices with 2GB RAM and slower CPUs.

## Purpose

Provides compatibility bridge for legacy Android devices (SDK 24+) that cannot run the latest BVR/BVI kit apps. Uses simpler sensor protocols and lower resource overhead. Sunset planned post-Wave 3 election operations.

## Features

- Legacy ARM32 and ARM64 CPU support
- Minimal resource footprint (low RAM, CPU requirements)
- gRPC shim protocol for backward compatibility
- Fingerprint biometric capture (iris optional based on device)
- Manual device certificate signing
- 7-day encrypted offline queue
- 2-language UI (Swahili, English)
- WCAG 2.2 AA (best-effort on small screens)

## Build

```bash
cd services/biometric-edge-svc/apps/legacy-android
flutter build apk --release --target-platform android-arm --flavor legacy
flutter build apk --split-per-abi --flavor legacy
```

## Development

```bash
flutter pub get
flutter run --flavor development
```

## Tests

```bash
flutter test
# Integration tests require physical legacy device or legacy emulator image
flutter test integration_test/
```

## Environment

Configure via `assets/config/`:
- `API_BASE_URL`: legacy shim adapter endpoint
- `SHIM_GRPC_HOST`: shim gRPC service hostname
- `SHIM_GRPC_PORT`: shim gRPC service port
- `DEVICE_CERT_PATH`: path to device certificate

## Device Requirements

- Android SDK 24 (API level 24) minimum
- 2GB RAM
- ARM32 or ARM64 processor
- Legacy fingerprint sensor (iris not guaranteed)
- Manual APK sideload via MDM (devices not compatible with Jamf)

## Release

Manual APK distribution via MDM sideload. Not available on public app stores.

## Architecture

- **Frontend**: Flutter + Dart, legacy sensor integration
- **Backend**: gRPC client connecting to legacy-android-shim integration
- **Storage**: SQLite encrypted queue
- **Attestation**: basic device cert (Play Integrity not available)

## Limitations

- Play Integrity attestation not available on legacy devices
- Sensor SDK coupling to legacy hardware
- No automatic app updates (manual sideload)
- Limited platform support (Android only)

## Known Issues

- Slower biometric matching on ARM32 processors
- May require manual sensor calibration
- Battery drain higher than modern kit apps

## Related Services

- **legacy-android-shim**: gRPC shim adapter (integrations/legacy-android-shim)
- **biometric-edge-svc**: modern API (not directly compatible)

## Timeline

Sunset: post-Wave 3 (2026 post-election cleanup). Device inventory planned for refurbishment or secure disposal.
