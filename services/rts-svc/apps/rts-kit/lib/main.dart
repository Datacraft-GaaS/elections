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
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:logging/logging.dart';

import 'src/rts_workflow/mode_switch.dart';
import 'src/rts_workflow/queue.dart';
import 'src/app.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await TelemetryService.init(
    appId: 'rts-kit',
    personaRole: 'results_transmission_officer',
  );

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

  runEelGuardedApp(
    app: ProviderScope(
      overrides: [
        outboxQueueProvider.overrideWithValue(queue),
        modeBundleProvider.overrideWith((ref) => modeBundle),
      ],
      child: const EelRtsKitApp(),
    ),
  );
}
