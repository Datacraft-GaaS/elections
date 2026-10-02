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

/// Diaspora Kit - embassy station biometric capture and pouch management (WP-5.8, M24).
///
/// Workflow:
///   1. Select open embassy station from local Isar cache.
///   2. Choose import profile (nairobi/london/dubai/washington/rest-of-world).
///   3. Capture Form 34A results (candidate tallies, rejected, disputed).
///   4. Sign with kit Ed25519 key and queue for delayed transmission.
///   5. Manage physical pouches: seal, transfer, receive.

import 'package:eel_kit_sdk/eel_kit_sdk.dart';
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
