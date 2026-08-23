/// Minimal smoke-test entry point for the eel RTS Kit.
///
/// Launches the app with in-memory providers. Run with:
///   flutter run -d <device> -t example/main.dart
library;
import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:eel_rts_kit/src/app.dart';
import 'package:eel_rts_kit/src/rts_workflow/mode_switch.dart';
import 'package:eel_rts_kit/src/rts_workflow/queue.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  final storage = InMemoryOutboxStorage();
  final queue = EelOfflineQueue(storage: storage);
  await queue.init();

  final modeBundle = await RtsModeBundle.loadCurrent();

  runApp(
    ProviderScope(
      overrides: [
        outboxQueueProvider.overrideWithValue(queue),
        modeBundleProvider.overrideWith((ref) => modeBundle),
      ],
      child: const EelRtsKitApp(),
    ),
  );
}
