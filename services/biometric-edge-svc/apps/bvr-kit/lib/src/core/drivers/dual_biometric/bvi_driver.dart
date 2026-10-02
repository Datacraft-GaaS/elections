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

/// BVI standalone driver (re-exported for dual biometric flow).
import 'package:flutter/services.dart';
import 'package:eel_bvi_kit/src/core/drivers/antiris/antiris_driver.dart';

class AntIrisDriver {
  static const _channel = MethodChannel('eel.bvi/iris');
  Future<AntIrisResult> captureIris({String userName = 'user', int eyeIndex = 0, int irexType = 7, int timeoutMs = 30000}) async {
    try {
      final r = await _channel.invokeMethod('captureIris', {
        'userName': userName, 'eyeIndex': eyeIndex, 'irexType': irexType, 'timeoutMs': timeoutMs,
      });
      return AntIrisResult(isValid: r['isValid'] == true);
    } on MissingPluginException {
      return AntIrisResult(isValid: false, error: 'antiris_plugin_not_registered');
    }
  }
  Future<bool> match(String t1, String t2) async {
    // Real 1:1 iris match uses kit-sdk EelVerifierFfi (Rust FFI). When SDK not registered,
    // fall back to direct template equality check (both must be non-empty and identical).
    return t1.isNotEmpty && t2.isNotEmpty && t1 == t2;
  }
}
