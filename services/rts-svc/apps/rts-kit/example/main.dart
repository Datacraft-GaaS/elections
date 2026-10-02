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
