/// kit-l1 on-device IsoForest anomaly plugin entry point.
///
/// This package is embedded as a Flutter plugin inside the eel platform.
library;
/// The main() here is the standalone test/dev runner; in production the plugin
/// is initialised by the host app.

import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter/material.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await TelemetryService.init(
    appId: 'kit-l1',
    personaRole: 'anomaly_detection_plugin',
  );
  // Standalone runner has no UI; telemetry runs in background.
  runApp(const SizedBox.shrink());
}
