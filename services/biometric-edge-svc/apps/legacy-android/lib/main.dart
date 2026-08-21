/// Legacy Android parity shim (WP-3.8).
///
/// Targets Android 8+ (API 26) minimum SDK.
/// Feature flag: eel_legacy_android_shim (via integrations/legacy-android-shim).
/// Attestation tier: software-only; backend marks attestation_tier=legacy.
/// See PARITY.md for feature-by-feature availability matrix.

import 'package:flutter/material.dart';

import 'src/app.dart';

void main() {
  runApp(const LegacyAndroidApp());
}
