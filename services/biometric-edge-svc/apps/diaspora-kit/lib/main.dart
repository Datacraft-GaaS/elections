/// Diaspora Kit - embassy station biometric capture and pouch management (WP-5.8, M24).
///
/// Workflow:
///   1. Select open embassy station from local Isar cache.
///   2. Choose import profile (nairobi/london/dubai/washington/rest-of-world).
///   3. Capture Form 34A results (candidate tallies, rejected, disputed).
///   4. Sign with kit Ed25519 key and queue for delayed transmission.
///   5. Manage physical pouches: seal, transfer, receive.

import 'package:eel_kit_sdk/telemetry.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'src/app.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await TelemetryService.init(
    appId: 'diaspora-kit',
    personaRole: 'diaspora_enrolment_officer',
  );
  runApp(
    const ProviderScope(
      child: DiasporaKitApp(),
    ),
  );
}
