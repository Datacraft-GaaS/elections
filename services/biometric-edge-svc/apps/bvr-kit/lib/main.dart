/// BVR Kit — biometric voter registration application (WP-3.8, M01).
///
/// Enrolment workflow:
///   1. Capture biometrics (fingerprint / iris / face) with quality gates.
///   2. MRZ scan of ID document.
///   3. IPRS pre-verify (online or deferred if offline).
///   4. Sign capture payload with device Ed25519 key.
///   5. Upload to edge-api /enrolment/upload -> bvr-svc.
///   6. Print QR handoff slip for voter.
///
/// Sustained target: >= 12 enrolments/hr.
/// Supported paths: standard, degraded-finger, amputee/assisted (EC-HUM-02,
/// EC-HUM-05), distress finger (EC-IDN-08), s.44A dual-officer (EC-HUM-01).
/// 11-language UI via packages/i18n.

import 'package:flutter/material.dart';
import 'package:eel_kit_sdk/telemetry.dart';

import 'src/app.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await TelemetryService.init(
    appId: 'bvr-kit',
    personaRole: 'enrolment_officer',
  );
  runApp(const BvrKitApp());
}
