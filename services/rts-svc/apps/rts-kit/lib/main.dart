import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:logging/logging.dart';

import 'src/rts_workflow/mode_switch.dart';
import 'src/rts_workflow/queue.dart';
import 'src/app.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  Logger.root.level = Level.INFO;
  Logger.root.onRecord.listen((rec) {
    // Production: forward to structured log sink. Dev: stderr.
    debugPrint('[${rec.level.name}] ${rec.loggerName}: ${rec.message}');
  });

  final storage = SqliteOutboxStorage();
  await storage.open();

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
