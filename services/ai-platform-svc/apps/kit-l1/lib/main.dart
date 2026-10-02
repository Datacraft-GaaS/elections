// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
