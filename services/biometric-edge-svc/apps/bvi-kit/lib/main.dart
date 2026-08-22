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

import 'package:flutter/material.dart';
import 'package:eel_kit_sdk/telemetry.dart';

import 'src/app.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await TelemetryService.init(
    appId: 'bvi-kit',
    personaRole: 'identification_officer',
  );
  runApp(const BviKitApp());
}
