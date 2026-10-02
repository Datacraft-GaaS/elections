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

/// Legacy Android parity shim (WP-3.8).
///
/// Targets Android 8+ (API 26) minimum SDK.
/// Feature flag: eel_legacy_android_shim (via integrations/legacy-android-shim).
/// Attestation tier: software-only; backend marks attestation_tier=legacy.
/// See PARITY.md for feature-by-feature availability matrix.

import 'package:flutter/material.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

import 'src/app.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await TelemetryService.init(
    appId: 'legacy-android',
    personaRole: 'enrolment_officer',
  );
  runApp(const LegacyAndroidApp());
}
