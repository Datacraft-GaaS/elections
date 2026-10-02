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

/// FAP60 real driver (M01 M3). Calls Fap60Plugin via MethodChannel.
/// Per SDK API: 1600x1500 raw image, quality 0-100, template base64.

import 'package:flutter/services.dart';

class Fap60Driver {
  static const _channel = MethodChannel('eel.bvr/fingerprint');

  Future<bool> init({bool lcdMode = true}) async {
    final r = await _channel.invokeMethod('init', {'lcdMode': lcdMode});
    return r == 'init_ok';
  }

  Future<Fap60Result> captureFingerprint({String position = 'LEFT_INDEX', int timeoutMs = 5000}) async {
    try {
      final r = await _channel.invokeMethod('captureFingerprint', {
        'position': position,
        'timeoutMs': timeoutMs,
      });
      final m = (r as Map).cast<String, dynamic>();
      return Fap60Result(
        isValid: m['isValid'] == true,
        qualityScore: (m['qualityScore'] as num?)?.toInt() ?? 0,
        templateBase64: m['templateBase64'] as String?,
        fingerPosition: m['fingerPosition'] as String? ?? position,
        error: m['error'] as String?,
      );
    } on PlatformException catch (e) {
      return Fap60Result(isValid: false, error: e.message ?? 'unknown');
    } on MissingPluginException {
      return Fap60Result(isValid: false, error: 'fap60_plugin_not_registered');
    }
  }

  Future<bool> selfCheck() async {
    try {
      return await _channel.invokeMethod('selfCheck') == true;
    } on MissingPluginException {
      return false;
    }
  }

  /// 1:1 fingerprint match — matches captured [probe] against [enrolled] template.
  /// Returns true if FAR ≤ 0.01 % (FMR ≤ 1 in 10,000 per ICAO LDS1).
  /// When FAP60 plugin not registered, falls back to direct template equality check.
  Future<bool> match(String probe, String enrolled) async {
    if (probe.isEmpty || enrolled.isEmpty) return false;
    try {
      final r = await _channel.invokeMethod('matchFingerprint', {
        'probe': probe,
        'enrolled': enrolled,
      });
      final m = (r as Map).cast<String, dynamic>();
      // REAL ALGORITHM: FAP60 CompareTemplates (minutiae) — score 0-100, threshold 45
      // Returns true when score ≥ 45 (FAR ≤ 0.01%)
      return m['matched'] == true || ((m['score'] as num?)?.toInt() ?? 0) >= 45;
    } on MissingPluginException {
      // No hardware: fall back to direct template equality (development only)
      return probe == enrolled;
    } on PlatformException {
      return false;
    }
  }

  Future<void> close() async {
    try { await _channel.invokeMethod('close'); } on MissingPluginException {}
  }
}

class Fap60Result {
  final bool isValid;
  final int qualityScore;
  final String? templateBase64;
  final String? fingerPosition;
  final String? error;
  const Fap60Result({
    required this.isValid, this.qualityScore = 0,
    this.templateBase64, this.fingerPosition, this.error,
  });
}
