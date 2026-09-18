# Factory Test Harness

This Flutter application is designed to exercise all major hardware and connectivity features of the Eel devices for factory validation and debugging. It runs tests sequentially, collects timing information, device details, and produces a detailed JSON report.

## Features Tested

- **Iris Scanner** – via method channel `eel.bvi/iris`
- **Fingerprint Scanner** – via method channel `eel/fingerprint`
- **Printer** – via method channel `eel.bvi/printer`
- **SMS Modem** – via method channel `eel/modem/sms`
- **Document Scanner** – via method channel `eel/bvi/document_scanner`
- **Camera** – via method channel `eel/bvi/camera`
- **GNSS** – using `geolocator` plugin
- **NTS** – HTTPS request to configured NTS endpoint
- **Battery** – using `battery_plus` plugin

On launch, the app gathers device information, then runs all tests automatically in sequence. Each test is timed, and results are logged to the screen. A detailed JSON report is saved to the app’s documents directory.

## Building the APK

```sh
cd factory_pack/test_harness
flutter pub get
flutter build apk --release   # or --debug for verbose logs
```

The output APK will be at:
`build/app/outputs/flutter-apk/app-release.apk`

A pre‑built release APK has been placed in the parent `factory_pack` directory as `factory_test_harness.apk`.

## Installation & Usage

1. **Install on target device**

   ```sh
   adb install -r factory_test_harness.apk
   ```

2. **Launch the app**

   ```sh
   adb shell monkey -p com.eel.test_harness -c android.intent.category.LAUNCHER 1
   ```

   (The package name is `com.eel.test_harness`; if changed, adjust accordingly.)

3. **Retrieve the test report**
   After the tests complete (a few seconds), pull the JSON report:

   ```sh
   adb shell run-as com.eel.test_harness cat app_flutter/test_report.json > test_report.json
   ```

   Or view it directly on the device:

   ```sh
   adb shell run-as com.eel.test_harness cat app_flutter/test_report.json
   ```

## Understanding the Report

The JSON report contains:

- `timestamp`: ISO‑8601 UTC time when the report was generated.
- `startTime` / `endTime`: ISO‑8601 timestamps for the start and end of the test suite.
- `overallDurationMs`: total time taken to run all tests.
- `deviceInfo`: information about the device (OS, version, hostname, etc.).
- `results`: an array of objects, one per feature, each with:
  - `feature`: name of the feature tested
  - `status`: `PASS`, `FAIL`, `SKIP`, or `ERROR`
  - `details`: human‑readable description of the outcome
  - `data`: optional raw data returned by the test (e.g., iris capture result, battery state)
  - `testStart` / `testEnd`: timestamps for when the test started and ended
  - `durationMs`: time taken to execute the test (in milliseconds)

A status of `SKIP` indicates the required plugin or native library was not found; `ERROR` indicates an exception; `FAIL` indicates the feature did not meet the success criteria.

## Customising the Test Harness

- To change the package name, edit `android/app/src/main/AndroidManifest.xml` and update the `package` attribute, then adjust the ADB commands accordingly.
- To modify test parameters (e.g., NTS URL, fingerprint quality threshold), edit the constants in `lib/main.dart`.
- To add additional tests, follow the pattern of the existing `_test*` methods, add them to the `tests` list in `_runAllTests`, and ensure they return a `Future<Map<String, dynamic>>`.

## Troubleshooting

- **MissingPluginException** – indicates that the method channel is not implemented on the native side. Ensure the corresponding native plugin (e.g., Antiris SDK, printer daemon) is present on the device.
- **Timeouts** – some tests wait for hardware responses; increase timeout values in the method‑channel calls if needed.
- **Permission errors** – the test harness requests location runtime permission; if denied, the GNSS test will fail.

## License

This test harness is part of the Eel platform and is subject to the same licensing as the rest of the codebase.
