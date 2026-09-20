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
