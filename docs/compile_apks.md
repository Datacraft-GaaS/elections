# How to Compile Factory-Testable APKs for BVI-Kit, BVR-Kit, RTS-Kit

This document summarizes the exact steps and environment needed to produce debug APKs for the three Flutter kits:

- `services/biometric-edge-svc/apps/bvi-kit`
- `services/biometric-edge-svc/apps/bvr-kit`
- `services/rts-svc/apps/rts-kit`

## Prerequisites

1. **Flutter SDK** (stable channel, version used: 3.47.1)  
   Ensure `flutter` is in your PATH and run `flutter doctor` to verify no issues.

2. **Java Development Kit**  
   Flutter 3.47.1 with Android Gradle Plugin (AGP) requires **Java 17**.  
   Install OpenJDK 17 (e.g., via Homebrew: `brew install openjdk@17`) and set it as the Flutter JDK:

   ```sh
   export JAVA_HOME="$(brew --prefix openjdk@17)/libexec/openjdk.jdk/Contents/Home"
   flutter config --jdk-dir="$JAVA_HOME"
   ```

3. **Gradle Version**  
   The projects require Gradle 8.11.1 (or higher) due to AGP constraints.  
   Verify the wrapper in each app’s `android/gradle/wrapper/gradle-wrapper.properties`:

   ```properties
   distributionUrl=https\://services.gradle.org/distributions/gradle-8.11.1-all.zip
   ```

   If you encounter version mismatches, edit the wrapper to the correct version.

4. **Android SDK**  
   Ensure `ANDROID_HOME` points to your Android SDK (typically `$HOME/Library/Android/sdk` on macOS).  
   Install platform‑tools, build‑tools, and Android 13 (API 33) or higher via `sdkmanager`.

5. **Dependencies**  
   Run `flutter pub get` in each kit’s directory before building.

## Common Build Flags

All kits require the following flags to avoid known issues:

- `--android-skip-build-dependency-validation` – bypasses Flutter’s Gradle version check (use if you see “Flutter's minimum supported version of 8.14.0” errors).
- `-Pandroid.newDsl=false` – disables the new Android DSL (required for AGP < 9.0, which these projects use).

## Step‑by‑Step Build Procedure

Repeat for each kit:

```sh
# 1. Change to the kit directory
cd services/biometric-edge-svc/apps/bvi-kit   # or bvr-kit, or rts-kit

# 2. Ensure dependencies are resolved
flutter pub get

# 3. (Optional) Verify no analysis issues
flutter analyze

# 4. Build the debug APK
flutter build apk --debug \
    --android-skip-build-dependency-validation \
    -Pandroid.newDsl=false
```

The resulting APK will be placed at:

```
build/app/outputs/flutter-apk/app-debug.apk
```

## Kit‑Specific Notes & Fixes Applied

### BVI‑Kit (`bvi-kit`)

- Required manual addition of missing getters in `lib/src/core/drivers/bvi_driver.dart`:
  - `double? get accuracy => accuracyMeters;`
  - `double? get lon => lng;`
  - `bool get fixed => lat != null && lng != null && error == null;`
  - `DateTime? get capturedAt { ... }` (parses `serverTimestamp`).
- The file `lib/src/core/drivers/bvi_driver.dart` originally lacked these getters, causing compile errors in `identification_workflow.dart`.

### BVR‑Kit (`bvr-kit`)

- Needed to extend `SensorModality` enum with `mrz` in `lib/src/sensor_sdk.dart`.
- Added a `factory CaptureResult.failure` constructor and an `error` field to `CaptureResult`.
- After changes, run `flutter pub get` to pick up the `isar_flutter_libs` transitive dependency (a Gradle namespace fix was applied globally to the cached package).

### RTS‑Kit (`rts-kit`)

- Fixed null‑safety in `lib/src/core/at_command/usb_serial_transport.dart`:
  - Changed `sub = _port!.inputStream.listen(...)` to `sub = _port!.inputStream!.listen(...)` after ensuring `_port` is non‑null.
  - Replaced invalid `device.createPort()` call with `device.port` (the `usb_serial` API uses a getter).
  - Removed the conditional import `usb_serial_for_android_plugin.dart` (not needed for desktop/Android hybrid builds; kept only the core import).
- Added a local `eel_design` package under `packages/eel_design` with a minimal pubspec and lib/eel_design.dart providing `EelScaffold`, `EelTextStyle`, `EelColors`, `EelResultCard`, and `EelButton`.
- Updated `pubspec.yaml` to reference the local path: `eel_design: {path: ../../../../packages/eel_design}`.
- Fixed abstract class instantiation in `lib/src/rts_workflow/sign.dart`:
  - Changed `(ref) => EelAttestationClient()` to `(ref) => EelAttestationClientStub()` to use the concrete stub implementation for development.
- Updated Kotlin version in `android/settings.gradle.kts` from `2.0.21` to `2.2.0` to resolve Kotlin metadata version conflicts.

## Verification

To confirm the APK is installable:

```sh
adb install -r build/app/outputs/flutter-apk/app-debug.apk
adb shell pm list packages | grep eel   # verify package name appears
```

## Clean Build

If you encounter stale artifacts, run:

```sh
flutter clean
flutter pub get
```

then rebuild.

## Generated APK Locations

After successful build, the APKs can be found at:

- **bvi-kit**: `/Users/nyimbiodero/src/pjs/eel/services/biometric-edge-svc/apps/bvi-kit/build/app/outputs/flutter-apk/app-debug.apk`
- **bvr-kit**: `/Users/nyimbiodero/src/pjs/eel/services/biometric-edge-svc/apps/bvr-kit/build/app/outputs/flutter-apk/app-debug.apk`
- **rts-kit**: `/Users/nyimbiodero/src/pjs/eel/services/rts-svc/apps/rts-kit/build/app/outputs/flutter-apk/app-debug.apk`

For convenience, copies have been placed in:

- `/Users/nyimbiodero/src/pjs/eel/target/mobile/bvi-kit-debug.apk`
- `/Users/nyimbiodero/src/pjs/eel/target/mobile/bvr-kit-debug.apk`
- `/Users/nyimbiodero/src/pjs/eel/target/mobile/rts-kit-debug.apk`

---
*This note captures the exact workflow discovered on 2025‑09‑16 to avoid re‑investigation.*
