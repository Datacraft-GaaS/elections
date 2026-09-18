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
  - Kept `await device.createPort()` with an explicit null check (`throw StateError` when null) — the earlier note about replacing it with `device.port` was inaccurate; createPort is the valid API on this `usb_serial` version.
  - Removed the conditional import `usb_serial_for_android_plugin.dart` (not needed for desktop/Android hybrid builds; kept only the core import).
- Added a local `eel_design` package under `packages/eel_design` with a minimal pubspec and lib/eel_design.dart providing `EelScaffold`, `EelTextStyle`, `EelColors`, `EelResultCard`, and `EelButton`.
- Updated `pubspec.yaml` to reference the local path: `eel_design: {path: ../../../../packages/eel_design}`.
- Fixed abstract class instantiation in `lib/src/rts_workflow/sign.dart`:
  - Changed `(ref) => EelAttestationClient()` to `(ref) => EelAttestationClientStub()` to use the concrete stub implementation for development.
- Updated Kotlin version in `android/settings.gradle.kts` from `2.0.21` to `2.2.0` to resolve Kotlin metadata version conflicts.

## Release builds (2026-09-18)

Release (AOT, non-debuggable) APKs built with the same flags plus `--release`:

| App | Size | Budget | Cold start (emulator) | Smoke |
|---|---|---|---|---|
| bvi-kit-release.apk | 50.6 MB | ≤ 60 MB ✅ | 450 ms | PASS, crash-free |
| bvr-kit-release.apk | 50.6 MB | ≤ 60 MB ✅ | 485 ms | PASS, crash-free |
| rts-kit-release.apk | 61.3 MB | ≤ 60 MB ⚠️ +2% over | 573 ms | PASS, crash-free |

SHA-256:

- `bvi-kit-release.apk`: `836c5cb1…87b79b1f85`
- `bvr-kit-release.apk`: `995d5b66…f3771426b`
- `rts-kit-release.apk`: `813c1042…0414af958`

Release notes and caveats:

- **Signed with the debug key** (`signingConfigs.getByName("debug")` in all
  three `android/app/build.gradle.kts`; no `key.properties` exists). Verified
  non-debuggable (`aapt` shows no `application-debuggable`) and v2-signed via
  `apksigner verify`. These validate the release *build path* — they are NOT
  production-signed. Real signing is M5 work (`sign.sh` + OpenBao).
- **isar compileSdk fix (pub-cache surgery):** release builds fail in
  `:isar_flutter_libs:verifyReleaseResources` because the cached
  `isar_flutter_libs-3.1.0+1` pins `compileSdkVersion 30` while referencing
  `android:attr/lStar` (API 31+). Fixed by bumping the cached package to
  `compileSdkVersion 35` (same category as the namespace fix already applied
  there). ⚠️ This edit lives **outside version control**: any `pub cache
  repair`, cache clean, or fresh CI machine loses it and release builds break
  again. Durable options (M5/CI): a post-`pub get` patch step, a forked
  `isar_flutter_libs` via dependency override, or replacing isar. Debug builds
  are unaffected (app-level linking covers it).
  An app-side `android.disableResourceValidation=true` was tried first and did
  NOT suppress the task on this AGP — reverted.
- **rts-kit is 2% over budget** (61.3 MB decimal; 58.5 MiB). The delta vs the
  other kits is `libapp.so` (rts bundles ~2× the Dart code). Remedies if the
  gate must be strictly met: `--split-per-abi` release (arm64-only ≈ 35 MB)
  or a libapp diet during M2. The `build.sh` size gate measures MiB and
  currently passes rts (58 MiB) — the plan should state the unit explicitly.
- **rts-kit requests CAMERA + RECORD_AUDIO at startup.** On a fresh install
  the system grant dialog wedged the task and swallowed all subsequent
  launches (indistinguishable from "app won't start" headlessly). Always
  install with `adb install -r -g` (now default in our scripts) or grant via
  MDM managed config. Privacy/factory note: document why a reporting app
  needs microphone at first launch.

## Traceability (which source produced these APKs)

> ⚠️ The APKs in this directory were built from a **dirty working tree on top
> of commit `3f9fa2af`**, not from a clean commit. Do not treat them as
> release artifacts. The exact mutations are listed below; land each as a
> reviewed change before any sign-off build.

Uncommitted modifications at build time (verify with `git status --porcelain`):

- `M services/biometric-edge-svc/apps/bvi-kit/lib/src/core/drivers/bvi_driver.dart`
- `M services/biometric-edge-svc/apps/bvr-kit/lib/src/sensor_sdk.dart`
- `M services/rts-svc/apps/rts-kit/lib/src/core/at_command/usb_serial_transport.dart`
- `M services/rts-svc/apps/rts-kit/lib/src/rts_workflow/sign.dart`
- `M services/rts-svc/apps/rts-kit/pubspec.yaml`
- `M services/rts-svc/apps/rts-kit/android/settings.gradle.kts`
- `?? packages/eel_design/` (untracked stub package — see divergence note)

SHA-256 of the shipped binaries (also in `SHA256SUMS` in the zip).
Rebuilt 2026-09-18 from the same dirty tree (see mutations above) with
Flutter 3.47.1, verified on a booted emulator (see validation log below):

- `bvi-kit-debug.apk` (180 MB): `7b5467fb…5379da661`
- `bvr-kit-debug.apk` (180 MB): `4968fcbc…ceae5878ad`
- `rts-kit-debug.apk` (160 MB): `81d3320c…8911aa69d`
- `factory_test_harness.apk` (166 MB): rebuilt 2026-09-18 from
  `factory_pack/test_harness` with applicationId `com.eel.test_harness`
  `factory_pack/test_harness` with applicationId `com.eel.test_harness`
  (renamed from `com.example.test_harness`; MainActivity moved to
  `com/eel/test_harness/`, namespace + applicationId updated in
  `android/app/build.gradle.kts`). Debug-signed (v2 scheme), debuggable
  (required for `run-as` report retrieval). SHA-256:
  `6e3f8e20…0e25569`. iOS/desktop platform dirs still carry example bundle
  IDs — out of scope (factory is Android-only; zip excludes those dirs).

Known divergences to resolve (Wave M2 app-migration work, not factory fixes):

- `packages/eel_design/` is a **local stub** (`EelScaffold` with `title/variant/padding/body`,
  `EelTextStyle`, `EelColors.of(context)`) that does **not** match the real M0.1
  package `packages/design-system/flutter` (different `EelScaffold` signature, no
  `EelTextStyle`, `EelColors` is a `ThemeExtension`). Only `rts-kit` depends on the
  stub and its screens use the stub API — swapping the dependency requires
  rewriting those screens. Tracked as M2 migration work; until then the stub is
  pinned build input, not to be extended.
- `bvr-kit/lib/src/ui/biometric_outbox_panel.dart` imports `package:eel_design`
  (undeclared in its pubspec) but is **unreferenced dead code** — harmless for
  `flutter build` (unreachable from `main.dart`) but flagged by
  `flutter analyze`. Delete the file or wire the dependency during M2.

## Verification

### Emulator validation log (2026-09-18, `eel-factory` AVD, API 35 arm64)

Environment had no emulator — installed via
`sdkmanager "emulator" "system-images;android-35;google_apis;arm64-v8a"`,
created AVD `eel-factory`, symlinked `system-images/` + `emulator/` into
`~/Library/Android/sdk` (SDK-root fix). Emulator left running as
`emulator-5554`; stop with `adb -s emulator-5554 emu kill`.

Results with the fixed `scripts/mobile/` drills:

| Check | Result |
|---|---|
| `smoke.sh bvi-kit` | PASS — alive (pid 4317), crash-free, cold start 2535 ms (WARN: emulator, budget is for physical hardware) |
| `smoke.sh bvr-kit` | PASS — alive, crash-free, cold start 2881 ms |
| `smoke.sh rts-kit` | PASS — alive, crash-free, cold start 2562 ms |
| `kill-drill.sh bvi-kit` | PASS — all 7 FSM stages: force-stop → new pid → no FATAL |
| `offline-drill.sh bvi-kit` | PASS — airplane toggle verified both ways, alive offline, survived reconnect (dedup SKIP: no backend hook) |
| `test_harness.sh` | Report retrieved; 6 hardware SKIP (no plugins on emulator), GNSS FAIL (30 s fix timeout — the unbounded hang is fixed), NTS ERROR (`time.eel.local` unresolvable here), Battery PASS |

Bugs found and fixed during validation (all in tree):

- `test_harness.sh` read the wrong JSON key (`tests` vs actual `results`) — would have reported false-pass.
- Report pulled from `files/test_report.json`; the app writes `app_flutter/test_report.json` — retrieval always failed before.
- Harness GNSS test had no timeout — hung forever on fix-less devices, no report ever written. Now 30 s → FAIL.
- `test_harness.sh` wait was 60 s; suite needs ~45 s + cold start → now 120 s.
- `eel_resolve_pkg` didn't match hyphenated slugs (`bvi-kit` vs `com.eel.bvi_kit`) — all drills failed to resolve. Now matches both spellings.
- `test_harness.sh` pack path was `../factory_pack` (wrong from `scripts/mobile`) — now `../../factory_pack`.
- `adb install -r` without `-g` left the harness blocked behind a permission dialog — now `-g` (also in `factory_test.sh` APK install).

Observations for app teams (not fixed — M2 work):

- `eel_kit_sdk` `location_service.dart` throws unhandled async exceptions when
  the app manifest lacks location permissions (seen on bvi-kit/rts-kit
  logcat); apps survive but the log noise hides real errors. Declare the
  permission or guard the call.
- `rts-kit` requests runtime permissions on startup (a system grant dialog
  appeared for it during testing) — on factory devices this blocks first
  launch; pre-grant via `adb install -g` (now default in our scripts) or
  move the request behind first use.

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

## Size note: debug (~160 MB) vs the 60 MB release budget

The shipped debug APKs are ~157–167 MB. This is **expected for debug builds**
and does NOT by itself violate the plan's ≤ 60 MB kit budget (plan §7/§9 apply
to release artifacts). Measured breakdown of `bvi-kit-debug.apk` (uncompressed):

| Payload | Size | In release? |
|---|---|---|
| `assets/flutter_assets/kernel_blob.bin` (debug JIT kernel) | ~77 MB | **No** — replaced by AOT snapshot (~5–10 MB) |
| `lib/x86_64/libflutter.so` (emulator ABI) | ~40 MB | **No** — per-ABI splits / arm-only factory builds |
| `lib/arm64-v8a/libVkLayer_khronos_validation.so` | ~15 MB | **No** — debug-only validation layer |
| `assets/flutter_assets/isolate_snapshot_data` | ~11 MB | **No** (debug) |
| `classes.dex` | ~10 MB | Yes |
| `libflutter.so` arm64 + armeabi-v7a | ~72 MB uncompressed | Yes, but release engine is smaller and usually single-ABI |
| `libisar.so` × ABIs | ~3 MB | Yes |

≈ 140 MB of the ~160 MB is debug-only. A release arm64-only build should land
well under 60 MB — enforced by the size gate in `scripts/mobile/build.sh`
(fails the build when over budget; override via `EEL_SIZE_BUDGET_MB`).
Note: kit APKs do **not** bundle the Antiris `.so`s — iris support requires the
native libs on the device library path (see README "Native Dependencies").

---
*This note captures the exact workflow discovered on 2025‑09‑16 to avoid re‑investigation.*
