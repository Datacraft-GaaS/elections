import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await TelemetryService.init(
    appId: 'lms-offline',
    personaRole: 'training_officer',
  );
  runApp(
    const ProviderScope(
      child: EelLmsOfflineApp(),
    ),
  );
}
