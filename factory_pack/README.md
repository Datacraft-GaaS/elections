# Factory Pack

This directory contains the artifacts needed for factory validation and testing of the Eel platform applications (BVI Kit, BVR Kit, RTS Kit) and the comprehensive test harness.

## Prerequisites

To use the contents of this factory pack, the following software must be installed on the factory workstation or testing environment:

- **Android SDK Platform Tools** – provides `adb` (Android Debug Bridge) for installing APKs and communicating with devices.
  - Install via Android Studio or download the command-line tools from <https://developer.android.com/studio/releases/platform-tools>
  - Ensure `adb` is in your `PATH`.

- **Java Development Kit (JDK)** – required for Android development and to run some build tools.
  - JDK 11 or newer is recommended.

- **Flutter SDK** – only needed if you wish to rebuild the test harness APK from source.
  - Install Flutter 3.19.0 or newer from <https://flutter.dev/docs/get-started/install>
  - Run `flutter doctor` to verify the installation.
  - The test harness APK is already pre‑built; Flutter is not required to run the tests on a device.

- **Antiris SDK** – the factory pack includes the necessary JAR files and native libraries (see the `antiris/` directory). If you need to rebuild the Antiris plugin, you will need the original Antiris SDK; however, the provided artifacts are sufficient for the test harness to function when the device has the native libraries available in its library path (or they are bundled with the APK).

## Contents

### Debug APKs

- `bvi-kit-debug.apk` – Debug build of the BVI Kit application.
- `bvr-kit-debug.apk` – Debug build of the BVR Kit application.
- `rts-kit-debug.apk` – Debug build of the RTS Kit application.

These APKs can be installed on target devices for manual testing or used with the factory test scripts.

### Test Harness APK

- `factory_test_harness.apk` – A specialized Flutter application that exercises all major hardware and connectivity features:
  - Iris Scanner
  - Fingerprint Scanner
  - Printer
  - SMS Modem
  - Document Scanner
  - Camera
  - GNSS (via geolocator plugin)
  - NTS (HTTPS endpoint)
  - Battery (via battery_plus plugin)
  - USB device enumeration
  - Bluetooth adapter and paired devices enumeration
  - Telephony (IMEI, SIM, network) information

  On launch, the harness runs all tests sequentially, collects timing and device information (including USB, Bluetooth, and telephony details), and outputs a detailed JSON report to the app’s documents directory.

### Documentation

- `factory_guidance.md` – Comprehensive guide on what to verify about an Android APK before release, including identity, cryptographic integrity, size, components, native libraries, resources, dependencies, security, testability, performance, and external native dependencies (e.g., Antiris SDK).
- `compile_apks.md` – Instructions on how to compile the APKs for the three applications.

### Scripts (under `scripts/mobile/`)

- `factory_test.sh` – Master test script that runs smoke, kill-drill, offline-drill, and the test harness sequentially.
- `smoke.sh` – Basic installation and launch test.
- `kill-drill.sh` – Stress test that repeatedly opens and closes the application.
- `offline-drill.sh` – Tests the application’s behavior when offline or with intermittent connectivity.
- `test_harness.sh` – Installs and runs the test harness APK, retrieves the JSON report, and validates that all tests passed.

### Native Dependencies

- `antiris/` – Contains the required Antiris SDK JAR files (`antiris.antusbcamera.jar`, `serenegiant.jar`) and native libraries (`.so` files for `armeabi-v7a` and `arm64-v8a`) needed for iris scanner functionality. If these are not present on the device, the test harness will report `SKIP` for iris and fingerprint tests.

### Test Harness Source (optional)

- The `test_harness/` directory contains the Flutter source code for the test harness application. It is included in the distribution package when the `--include-src` flag is used with the packaging script.

## Usage

### Quick Factory Test

```bash
# From the project root:
scripts/mobile/factory_test.sh <APP> --device <serial>
```

Replace `<APP>` with one of `bvi-kit`, `bvr-kit`, or `rts-kit` (the argument is currently ignored by the test harness script but required for interface consistency). Replace `<serial>` with the device’s ADB serial number (e.g., `emulator-5554` or a device ID from `adb devices`).

The script will:

1. Install the specified debug APK.
2. Run smoke, kill-drill, and offline-drill tests.
3. Install and run the test harness APK.
4. Pull and evaluate the JSON report from the test harness.
5. Exit with a non‑zero status if any step fails.

### Using the Test Harness Directly

```bash
# Install the test harness APK
adb install -r factory_test_harness.apk

# Launch the app (it will auto‑start the test sequence)
adb shell monkey -p com.example.test_harness -c android.intent.category.LAUNCHER 1

# Retrieve the JSON report after testing
adb shell run-as com.example.test_harness cat files/test_report.json > test_report.json
```

The JSON report contains:

- Overall test suite timing (`startTime`, `endTime`, `overallDurationMs`)
- Device information (`os`, `osVersion`, `hostname`, `model`, `manufacturer`, `usbDevices`, `bluetooth`, `telephony`)
- Per‑feature results with:
  - `feature`: name of the feature tested
  - `status`: `PASS`, `FAIL`, `SKIP`, or `ERROR`
  - `details`: human‑readable description
  - `data`: optional raw data (e.g., iris capture result, battery state)
  - `testStart` / `testEnd`: timestamps for the test
  - `durationMs`: time taken to execute the test (ms)

A status of `SKIP` indicates the required plugin or native library was not found; `ERROR` indicates an exception; `FAIL` indicates the feature did not meet the success criteria.

### Creating a Distribution Package

To create a zip file containing everything needed for the factory (APKs, scripts, documentation, native dependencies, and optionally the test harness source):

```bash
scripts/mobile/package_test_harness.sh --clean --zip --include-src
```

This will:

- Build (or copy) the test harness APK.
- Clean intermediate build artifacts (`flutter clean`).
- Generate `factory_test_harness_package.zip` in this directory.

The zip includes:

- All debug APKs (`bvi-kit-debug.apk`, `bvr-kit-debug.apk`, `rts-kit-debug.apk`)
- The test harness APK (`factory_test_harness.apk`)
- Documentation (`factory_guidance.md`, `compile_apks.md`)
- All relevant scripts (`factory_test.sh`, `smoke.sh`, `kill-drill.sh`, `offline-drill.sh`, `test_harness.sh`)
- The `antiris/` native dependencies
- Optionally, the full `test_harness/` source (with build artifacts excluded)

## License

The contents of this factory pack are part of the Eel platform and are subject to the same licensing as the rest of the codebase.
