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

import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import 'enrolment_workflow.dart';
import 'i18n_delegate.dart';
import 'sensor_sdk.dart';

class BvrKitApp extends StatelessWidget {
  const BvrKitApp({super.key, this.sensorDriver});

  /// Override for tests.
  final SensorDriver? sensorDriver;

  SensorDriver _resolveDriver() {
    if (sensorDriver != null) return sensorDriver!;
    // FAP60 driver works everywhere: on hardware it talks to `eel/fingerprint`,
    // on emulator/CI it falls back to synthetic captures (keeps CI green).
    if (!kIsWeb && defaultTargetPlatform == TargetPlatform.android) {
      return Fap60SensorDriver();
    }
    return const MockSensorDriver();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'eel BVR Kit',
      navigatorKey: eelNavigatorKey,
      localizationsDelegates: eelLocalizationDelegates,
      supportedLocales: eelSupportedLocales,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF006600)),
        useMaterial3: true,
      ),
      home: EnrolmentWorkflowScreen(sensor: _resolveDriver()),
    );
  }
}
