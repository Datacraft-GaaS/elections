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

/// AntIris iris scanner driver (M05 M3) — real SDK call.
/// Uses AntUSBCamera.jar (USBControl) for USB connection + capture.
/// SDK source: ./sdks/AntIris_Android_SDK/
/// Hardware: AntIris iris scanner (K7 capture mode, JP2/ISO output).
library;

import 'package:flutter/services.dart';

class AntIrisDriver {
  static const _channel = MethodChannel('eel.bvi/iris');

  Future<bool> init() async => await _channel.invokeMethod('init') == true;

  /// Capture iris image. Returns [AntIrisResult] with K7 raw image + JP2 compressed image.
  Future<AntIrisResult> captureIris({String userName = 'user', int eyeIndex = 0, int irexType = 7, int timeoutMs = 30000}) async {
    try {
      final r = await _channel.invokeMethod('captureIris', {
        'userName': userName,
        'eyeIndex': eyeIndex,
        'irexType': irexType,
        'timeoutMs': timeoutMs,
      });
      final m = (r as Map).cast<String, dynamic>();
      return AntIrisResult(
        isValid: m['isValid'] == true,
        imageBase64: m['imageBase64'] as String?,
        isoBase64: m['isoBase64'] as String?,
        width: (m['width'] as num?)?.toInt(),
        height: (m['height'] as num?)?.toInt(),
        quality: (m['quality'] as num?)?.toInt(),
        error: m['error'] as String?,
      );
    } on MissingPluginException {
      return const AntIrisResult(isValid: false, error: 'antiris_plugin_not_registered');
    } on PlatformException catch (e) {
      return AntIrisResult(isValid: false, error: e.message ?? 'antiris_unknown');
    }
  }

  Future<bool> selfCheck() async => await _channel.invokeMethod('selfCheck') == true;

  Future<void> stopCapture() async => await _channel.invokeMethod('stopCapture');
  Future<void> release() async => await _channel.invokeMethod('release');
}

class AntIrisResult {
  final bool isValid;
  final String? imageBase64;
  final String? isoBase64;
  final int? width;
  final int? height;
  final int? quality;
  final String? error;
  const AntIrisResult({
    required this.isValid,
    this.imageBase64,
    this.isoBase64,
    this.width,
    this.height,
    this.quality,
    this.error,
  });
}
