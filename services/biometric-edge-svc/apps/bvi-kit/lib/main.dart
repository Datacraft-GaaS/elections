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

/// BVI Kit — biometric voter identification application (WP-3.8, M05).
///
/// Identification workflow:
///   1. GNSS + IMU spoof detection (multi-constellation + IMU sanity envelope).
///   2. Time-fence FSM: refuse session if kit clock skew > 5 min (EC-TIM-01).
///   3. 1:1 multi-modal biometric match against enrolled reference.
///   4. BVI-svc identification event.
///   5. Ballot issue.
///
/// Offline: 7-day queue via kit-sdk offline_queue.
/// Boot target: <= 90 s.
/// Emergency paths: dual-control close (EC-OPS-04), s.44A (EC-LEG-03).
/// 11-language UI + a11y.
library;

import 'package:flutter/material.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

import 'src/app.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await TelemetryService.init(
    appId: 'bvi-kit',
    personaRole: 'identification_officer',
  );
  runEelGuardedApp(app: const BviKitApp());
}
