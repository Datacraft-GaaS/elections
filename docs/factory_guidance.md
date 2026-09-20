# Factory Guidance: What to Verify About an Android APK

This document consolidates the information a factory, QA, or release engineer typically wants to know about an Android APK before it is shipped to devices. Use it as a checklist when inspecting the APKs you have built (`bvi-kit-debug.apk`, `bvr-kit-debug.apk`, `rts-kit-debug.apk`) or when creating a release‑candidate build.

---  

## 1. Identity & Versioning  

| Item | Why it matters | How to check |
| ------ | ---------------- | -------------- |
| **Application ID** (`package` name) | Must match the one registered in the Play Store / internal distribution channel and the backend expectations. | `aapt dump badging <apk> \| grep package` |
| **Version Name** (`versionName`) | Human‑readable label shown to users; used in release notes and support tickets. | `aapt dump badging <apk> \| grep versionName` |
| **Version Code** (`versionCode`) | Integer that the OS uses to determine upgrade/downgrade; must increase with every release. | `aapt dump badging <apk> \| grep versionCode` |
| **Minimum / Target / Compile SDK** | Guarantees compatibility with the devices in the field and with the APIs you actually use. | `aapt dump badging <apk> \| grep sdkVersion:` or inspect `AndroidManifest.xml` |
| **Build timestamp / Git SHA** (if embedded) | Traceability to the exact source commit that produced the binary. | Look for a custom meta‑data field (e.g., `meta-data android:value="BUILD_TIME"`), or check `BuildConfig.VERSION_NAME` / `BuildConfig.GIT_SHA` if added. |

---  

## 2. Cryptographic Integrity  

| Item | Why it matters | How to check |
| ------ | ---------------- | -------------- |
| **Signing certificate SHA‑256** | Guarantees the APK hasn’t been tampered with and allows Play/App‑store updates. | `keytool -printcert -jarfile <apk>` |
| **Signing algorithm** (e.g., SHA256withRSA) | Some distribution channels require a specific algorithm. | Same `keytool` output. |
| **Multiple signatures** (apk‑scheme v1, v2, v3) | Modern Android requires at least v2; v3 adds APK Signature Scheme v3 for faster verification. | `apksigner verify --verbose <apk>` or `jar -tf <apk> \| grep META-INF` |
| **Debug vs. Release flag** | Debug builds are not signed with the release key and should never go to production. | Check if the signer is `Android Debug` (`CN=Android Debug,O=Android,C=US`). |

---  

## 3. Size & Compression  

| Item | Why it matters | How to check |
| ------ | ---------------- | -------------- |
| **Total APK size** | Influences download time, OTA update cost, and storage on low‑end devices. | `ls -lh <apk>` |
| **Raw (uncompressed) size** | Helps spot bloat from unused resources/libraries. | `unzip -l <apk> \| awk '{sum+=$1} END{print sum}'` |
| **Compression ratio** | Indicates whether resources are effectively compressed (PNG → WebP, etc.). | `(raw‑size / compressed‑size)*100` |
| **Presence of large assets** (e.g., raw images, video, native .so) | May need to be moved to asset bundles or downloaded on‑demand. | `unzip -l <apk> \| sort -k1 -nr \| head` |

---  

## 4. Components & Manifest  

| Item | Why it matters | How to check |
| ------ | ---------------- | -------------- |
| **Declared activities, services, receivers, providers** | Ensures no unintended entry points are exposed. | `aapt dump xmltree <apk> AndroidManifest.xml` |
| **Exported components** (`android:exported="true"` or implicit) | Exported components can be invoked by other apps – a security concern. | Look for `exported` true or missing (defaults to true for API<17). |
| **Permissions requested** | Must be justified; excess permissions raise flags in Play Store and user suspicion. | `aapt dump permissions <apk>` |
| **Uses‑features & uses‑libraries** | Confirms hardware requirements (e.g., `android.hardware.camera`, `android.hardware.fingerprint`). | `aapt dump features <apk>` |
| **Application class & meta‑data** | Custom `Application`, Firebase, etc., must be present and correctly configured. | Search `android:name` in `<application>` tag. |
| **Intent filters** | Verify that the app correctly handles deep links, share intents, etc. (if applicable). | Examine `<intent-filter>` nodes. |

---  

## 5. Native Libraries (if any)  

| Item | Why it matters | How to check |
| ------ | ---------------- | -------------- |
| **ABI support** (`armeabi-v7a`, `arm64-v8a`, `x86`, `x86_64`) | Determines which devices can run the APK; missing ABI leads to `INSTALL_FAILED_NO_MATCHING_ABI`. | `unzip -l <apk> \| grep lib/` |
| **Shared object size** | Large `.so` files bloat the APK; consider stripping or using the NDK’s `ANDROID_STL`. | `ls -lh <apk>/*/lib*.so` |
| **Stripping & symbol visibility** | Unstripped `.so` contain debug symbols, increasing size and potentially leaking IP. | `readelf -W -s <lib>.so \| grep -i debug` or `file <lib>.so`. |
| **Minimum Android version for each `.so`** | Some `.so` require a newer `minSdkVersion` than the manifest declares. | Check `android:minSdkVersion` in the library’s `AndroidManifest` (if bundled) or note the NDK used. |

---  

## 6. Resources & Asset Optimization  

| Item | Why it matters | How to check |
| ------ | ---------------- | -------------- |
| **Resource types** (drawable, layout, values, anim, etc.) | Unused resources increase APK size and can cause build warnings. | `aapt list <apk>` |
| **Drawable density buckets** (mdpi, hdpi, xhdpi, xxhdpi, xxxhdpi) | Ensure you are not shipping unnecessary densities (e.g., only xxxhdpi on low‑end devices). | `unzip -l <apk> \| grep res/drawable-` |
| **Vector vs. raster assets** | Vectors scale without extra size; prefer for icons. | Look for `.xml` in `drawable` folder. |
| **String localization** | Missing translations cause fallback to default language; extra locales increase size. | `unzip -l <apk> \| grep res/values-` |
| **Raw assets folder** (`assets/`) | Should contain only files truly needed at runtime (e.g., fonts, JSON configs). | `unzip -l <apk> \| grep assets/` |

---  

## 7. Dependency & Build Information  

| Item | Why it matters | How to check |
| ------ | ---------------- | -------------- |
| **Declared dependencies** (from `build.gradle`/`pubspec.yaml`) | Confirms you are using the intended versions of Flutter, plugins, and native SDKs. | Examine `pubspec.lock` or `build.gradle`. |
| **Transitive dependency versions** | Helps spot unintentionally upgraded libraries that could break ABI/API. | `flutter pub deps` or `./gradlew :app:dependencies`. |
| **Build flavor / variant** (if using product flavors) | Confirms you built the correct flavor (e.g., `staging` vs `production`). | Look at `base-application-name` or `applicationIdSuffix` in manifest. |
| **ProGuard / R8 rules** | Ensures the release build is properly obfuscated and that essential classes are not stripped. | Check `release` build output for `mapping.txt`; verify `-keep` rules. |
| **Flutter/Dart version** | Guarantees compatibility with plugins and the Dart VM shipped. | `flutter --version` or inspect `flutter_assets/version.json`. |

---  

## 8. Security & Privacy  

| Item | Why it matters | How to check |
| ------ | ---------------- | -------------- |
| **Clear‑text traffic** | Android 9+ blocks clear‑text HTTP unless explicitly allowed; could break network calls. | Look for `android:usesCleartextTraffic="true"` in `<application>` or network security config. |
| **Network Security Config** | Pinning, domain restrictions, etc., help mitigate MITM. | `res/xml/network_security_config.xml` (if present). |
| **Exported services/receivers with intent filters** | Could be abused for privilege escalation. | Review manifest for exported components with `<intent-filter>`. |
| **Debuggable flag** (`android:debuggable="true"`) | Must be `false` in release builds; enables JDWP and can leak data. | `aapt dump badging <apk> \| grep debuggable` |
| **Backup flag** (`android:allowBackup`) | If `true` and no custom backup agent, data may be extractable via ADB backup. | Same as above. |
| **Exported ContentProvider** | Must be protected (e.g., `android:exported="false"` or require a permission). | Check `<provider>` tags. |

---  

## 9. Testability & Instrumentation  

| Item | Why it matters | How to check |
| ------ | ---------------- | -------------- |
| **Test APK presence** (if you build a separate `*-test.apk`) | Allows running instrumentation tests on device. | `adb install -r <app>-test.apk` |
| **Flutter test coverage** (if you generate `lcov.info`) | Gives confidence that critical logic is exercised. | Look for `coverage/lcov.info` after `flutter test --coverage`. |
| **Screen‑shot / golden files** | Helps detect UI regressions automatically. | Check `test/goldens` folder. |
| **Ability to enable/disable feature flags via metadata or config** | Useful for factory diagnostics without rebuilding. | Look for `meta-data` tags or shared‑pref defaults. |

---  

## 10. Performance Indicators (optional, but useful for factory sign‑off)  

| Item | Why it matters | How to check (quick) |
| ------ | ---------------- | ---------------------- |
| **Cold‑start time** | Directly impacts user perception; should be < 2 s on target hardware. | `adb shell am start -S <pkg>/<activity>` and look at `TotalTime`. |
| **Frame‑rendering jank** | UI smoothness; >16 ms per frame leads to visible stutter. | `adb shell gfxinfo <pkg>` and look at `90th percentile` or `jank` frames. |
| **Memory usage (PSS)** | Ensures the app doesn’t exceed device memory limits under load. | `adb shell dumpsys meminfo <pkg>` |
| **CPU usage** | High CPU can drain battery and cause thermal throttling. | `adb shell top -m 5 -n 1 <pkg>` |
| **Battery drain** (rough) | Long sessions shouldn’t cause abnormal drain. | `adb shell dumpsys batterystats --charged <pkg>` (requires a full charge cycle). |

---  

## 11. External Native Dependencies (e.g., AntIris SDK)

| Item | Why it matters | How to check |
| ------ | ---------------- | -------------- |
| **Antiris SDK JAR files** (`antiris.antusbcamera.jar`, `serenegiant.jar`) | Required for iris scanner functionality via MethodChannel `eel.bvi/iris`. If missing, iris capture throws `MissingPluginException` and falls back to mock driver. | Look inside the APK: `unzip -l <apk> | grep antiris.antusbcamera.jar` or check `assets/` or `libs/` if bundled. If not present, the device must have the JAR pre‑installed in its classpath (e.g., via system image). |
| **Antiris native libraries** (`lib_Antiris.so`, `libUSBCamera.so`, `libopenjpeg.so`, `libpng.so`) for `armeabi-v7a` and `arm64-v8a` | Native code backing the Antiris JAR; must be present in the APK’s `lib/<abi>/` or on the device’s library path. | `unzip -l <apk> | grep lib_Antiris.so` etc. |
| **Verification** | Confirm that the APK either bundles these files or that the target device already provides them. | For a built APK, run the unzip commands above; if missing, you need to bundle them (e.g., place JAR in `src/main/assets` and load via DexClassLoader, or add as a compile‑time dependency in the plugin’s `android/` folder). |

---  

## 12. Factory Test Harness APK

A special APK (`test_harness.apk`) can be built to exercise all major hardware and connectivity features (fingerprint, iris, printer, SMS modem, document scanner, camera, GNSS, NTS, battery) and generate a detailed JSON report for debugging and validation.

**Where it lives**  
`factory_pack/test_harness/` contains a Flutter project that uses method channels to talk to the native subsystems (Antiris SDK for iris, fingerprint scanner, printer, SMS modem, document scanner, camera) and platform plugins (geolocator, battery_plus, http) to test each feature. It does **not** require the full eel application; it only needs the native libraries/JARs to be present on the device (or bundled) for the iris and fingerprint features to work.

**How to build**  

```sh
cd factory_pack/test_harness
flutter pub get
flutter build apk --release   # or --debug for verbose logs
```

The output APK will be at `build/app/outputs/flutter-apk/app-release.apk` (or debug).

**What it does**  
When launched, the test harness automatically runs the following tests in sequence and logs results to the screen and to a JSON report:

- **Iris**: Calls `eel.bvi/iris` method channel to init and capture; reports PASS/FAIL/SKIP/ERROR.
- **Fingerprint**: Uses the fingerprint method channel (`eel/fingerprint`) to init and capture; validates quality ≥ 40.
- **Printer**: Checks printer availability via `eel/bvi/printer` and sends a test page.
- **SMS Modem**: Checks SMS modem availability via `eel/modem/sms` and sends a test SMS.
- **Document Scanner**: Checks scanner availability via `eel/bvi/document_scanner` and attempts a scan.
- **Camera**: Checks camera availability via `eel/bvi/camera` and captures a test image.
- **GNSS**: Uses the `geolocator` plugin to request a location fix (low accuracy, fast).
- **NTS**: Attempts an HTTPS GET to a configurable NTS endpoint (default `https://time.eel.local/v1/time/nts`); you may need to adjust the URL or mock the server for offline testing.
- **Battery**: Uses `battery_plus` to retrieve state and level.

Each test logs to console and to an on‑screen list. After all tests complete, a JSON report is written to the app’s documents directory: `/data/data/<package>/files/test_report.json` (or accessible via `adb run-as <package> cat files/test_report.json`).

**Report format**  

```json
{
  "timestamp": "2025-09-14T16:52:00.000Z",
  "results": [
    { "feature": "Iris", "status": "PASS", "details": "...", "data": { ... } },
    { "feature": "Fingerprint", "status": "PASS", "details": "...", "data": { ... } },
    { "feature": "Printer", "status": "PASS", "details": "...", "data": { ... } },
    { "feature": "SMS Modem", "status": "PASS", "details": "...", "data": { ... } },
    { "feature": "Document Scanner", "status": "PASS", "details": "...", "data": { ... } },
    { "feature": "Camera", "status": "PASS", "details": "...", "data": { ... } },
    { "feature": "GNSS", "status": "PASS", "details": "...", "data": { ... } },
    { "feature": "NTS", "status": "PASS", "details": "...", "data": { ... } },
    { "feature": "Battery", "status": "PASS", "details": "...", "data": { ... } }
  ]
}
```

A status of `SKIP` indicates the required plugin or native library was not found; `ERROR` indicates an exception; `FAIL` indicates the feature did not meet success criteria.

**How to retrieve the report**  

```sh
# Assuming the test harness APK is installed with package name: com.example.test_harness
adb shell run-as com.example.test_harness cat files/test_report.json > test_report.json
# Or if the app stores it in external storage, adjust the path.
```

**Factory usage**  

1. Build the test harness APK (once per SDK change).  
2. Install it on the target device alongside the APK under test (or on a separate device).  
3. Run the test harness and collect the JSON report.  
4. Use the report to verify that each feature works with the current native libraries and configuration; any FAIL/ERROR/SKIP indicates a missing or mis‑configured dependency that must be addressed before releasing the main APK.

---

## Quick One-Liners to Gather Information

```sh
# Basic identity
aapt dump badging app-debug.apk | grep -E "package|versionName|versionCode|sdkVersion:"

# Signing info
apksigner verify --verbose app-debug.apk 2>&1 | grep "Signed by"
keytool -printcert -jarfile app-debug.apk | grep -E "SHA256|MD5|SHA1"

# Size breakdown
unzip -l app-debug.apk | awk '{sum+=$1} END {printf "Total uncompressed: %.2f MB\n", sum/1024/1024}'
ls -lh app-debug.apk

# Permissions
aapt dump permissions app-debug.apk

# Exported components
aapt dump xmltree app-debug.apk AndroidManifest.xml | grep -E "activity|service|receiver|provider" | grep -B2 -A2 "exported=true"

# Native libs
unzip -l app-debug.apk | grep lib/ | awk -F/ '{print $NF}' | sort | uniq -c

# Resources
aapt list app-debug.apk | head -20
```

You can script these checks into a **factory‑validation** step (e.g., a Jenkins/GitLab job) that fails the build if any required field is out of spec (wrong version code, debuggable=true, missing signature, oversized APK, etc.).

---  

### TL;DR – What the factory wants to know

1. **Who built it?** (package, version, build ID/Git SHA)  
2. **Is it genuine and untampered?** (signing cert, scheme, debug/release flag)  
3. **How big is it?** (total size, compression, native lib bloat)  
4. **What does it declare?** (activities, services, permissions, features)  
5. **Does it contain any dangerous bits?** (debuggable, clear‑text, exported components, over‑privileged permissions)  
6. **Are the dependencies correct and locked?** (Flutter version, plugin versions, NDK/API levels)  
7. **Are resources optimized?** (no unused assets, proper density buckets, vector drawables, localized strings)  
8. **Does it meet performance/security baselines?** (startup time, memory, jank, network config)  
9. **Can it be traced back to source?** (embedded build timestamp/commit hash)  
10. **Is it testable?** (unit/integration tests exist, coverage reports, ability to run smoke/kill/offline drills)  
11. **Are external native dependencies present?** (AntIris JAR and .so files for iris scanner; if missing, iris features will fail or fallback).  

Running the documented scripts (`smoke.sh`, `kill-drill.sh`, `offline-drill.sh`, and the orchestrated `factory_test.sh`) together with the manifest/APK inspection above gives the factory a comprehensive, automated “gate‑keeping” checklist before the APK is released to devices.  
