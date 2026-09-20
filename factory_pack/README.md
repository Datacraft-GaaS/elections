# Factory Pack

> **Installers and testers: start with [`INSTALLER_TESTER_GUIDE.md`](INSTALLER_TESTER_GUIDE.md)**
> (English + 中文 step-by-step instructions).
> **安装和测试人员请先读 [`INSTALLER_TESTER_GUIDE.md`](INSTALLER_TESTER_GUIDE.md)**（中英文步骤说明）。

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

These APKs can be installed on target devices for manual testing or used with the factory test scripts. Prefer debug builds for drill/test-harness flows (`run-as` report retrieval needs debuggable builds).

### Release APKs (factory validation of the release path)

- `bvi-kit-release.apk` (50.6 MB), `bvr-kit-release.apk` (50.6 MB),
  `rts-kit-release.apk` (61.3 MB — 2% over the 60 MB kit budget, see
  `compile_apks.md`).

AOT-compiled, non-debuggable, v2-signed — but signed with the **debug key**
(see `compile_apks.md`). They validate that the release build launches and
runs; they are **not production-signed** and must not go to MDM/production.
Install with `adb install -r -g <apk>` (`-g` pre-grants runtime permissions;
without it, rts-kit's startup CAMERA/RECORD_AUDIO request blocks headless
launches behind a system dialog).

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

- `INSTALLER_TESTER_GUIDE.md` – step-by-step for installers/testers (EN + 中文).
- `ENGINEERING_FAQ.md` – mass-production readiness Q&A for the engineering
  team (preinstalls, release signing, HAL/plugin delivery, Antiris placement).
- `factory_guidance.md` – Comprehensive guide on what to verify about an Android APK before release, including identity, cryptographic integrity, size, components, native libraries, resources, dependencies, security, testability, performance, and external native dependencies (e.g., Antiris SDK).
- `compile_apks.md` – Instructions on how to compile the APKs for the three applications.

### Scripts (in this directory — self-contained pack)

- `factory_test.sh` – Master test script: runs smoke, kill-drill, offline-drill, and test_harness sequentially; writes `factory-report-<APP>-<serial>.json`.
- `smoke.sh` – Cold-launch, assert process alive + crash-free, optional telemetry probe via `EEL_TELEMETRY_URL` (SKIP when unset, never faked).
- `kill-drill.sh` – `force-stop` at each FSM stage; asserts new pid + no FATAL EXCEPTION. Any failed stage fails the drill.
- `offline-drill.sh` – Verified airplane-mode toggle; app must stay alive offline and survive reconnect. Record-level dedup via `EEL_DEDUP_CHECK` hook (SKIP when unset, never a silent pass).
- `test_harness.sh` – Installs and runs the test harness APK, retrieves the JSON report, fails on FAIL/ERROR statuses.
- `package_test_harness.sh` – Rebuilds this zip (`--zip`), optionally with slimmed test-harness source (`--include-src`).
- `lib/common.sh`, `lib/device.sh` – Shared logging + adb helpers sourced by the scripts above. Kill-drill/offline-drill/test_harness require `lib/device.sh` beside them — it is included in this pack and in the zip.

### Native Dependencies

- `antiris/` – Contains the required Antiris SDK JAR files (`antiris.antusbcamera.jar`, `serenegiant.jar`) and native libraries (`.so` files for `armeabi-v7a` and `arm64-v8a`) needed for iris scanner functionality. If these are not present on the device, the test harness will report `SKIP` for iris and fingerprint tests.

### Test Harness Source (optional)

- The `test_harness/` directory contains the Flutter source code for the test harness application. It is included in the distribution package when the `--include-src` flag is used with the packaging script.

## Usage

### Quick Factory Test

```bash
# From this directory (factory_pack is self-contained):
./factory_test.sh <APP> --device <serial>
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
adb shell monkey -p com.eel.test_harness -c android.intent.category.LAUNCHER 1

# Retrieve the JSON report after testing
adb shell run-as com.eel.test_harness cat app_flutter/test_report.json > test_report.json
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
./package_test_harness.sh --zip --include-src
```

This will:

- Build the debug test harness APK (debug keeps `run-as` report retrieval working).
- Optionally clean intermediate build artifacts (`flutter clean` with `--clean`).
- Generate `factory_test_harness_package.zip` in this directory plus a `SHA256SUMS` manifest.

The zip includes:

- All debug APKs (`bvi-kit-debug.apk`, `bvr-kit-debug.apk`, `rts-kit-debug.apk`)
- The test harness APK (`factory_test_harness.apk`)
- `SHA256SUMS` — verify with `shasum -c SHA256SUMS` after unzipping
- Documentation (`factory_guidance.md`, `compile_apks.md`)
- All scripts (`factory_test.sh`, `smoke.sh`, `kill-drill.sh`, `offline-drill.sh`, `test_harness.sh`, `lib/common.sh`, `lib/device.sh`)
- The `antiris/` native dependencies
- Optionally, the slimmed `test_harness/` source (Android + shared sources; desktop/web/iOS platform dirs excluded)

## License

The contents of this factory pack are part of the Eel platform and are subject to the same licensing as the rest of the codebase.
