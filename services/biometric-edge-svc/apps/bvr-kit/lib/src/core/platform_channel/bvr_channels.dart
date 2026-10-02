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

/// Native MethodChannel definitions for bvr-kit (M01).
/// Plan.md M3: platform bridge to FAP60/document-camera/signature-pad native SDKs.
/// Hardware procurement: FAP60 + camera + touchscreen signature pad.
/// Falls back to graceful 'not_connected' on MissingPluginException (no hardware).

import 'package:flutter/services.dart';

/// BVR platform channel namespace.
abstract class BvrChannels {
  /// Fingerprint scanner (FAP60) — scan, capture, quality, cancel.
  static const fingerprint = MethodChannel('eel.bvr/fingerprint');

  /// Document camera — preview, snapshot, MRZ OCR (native).
  static const documentCamera = MethodChannel('eel.bvr/camera');

  /// Signature pad (touchscreen) — start, point, end, export.
  static const signature = MethodChannel('eel.bvr/signature');

  /// Hardware self-check — all subsystems status.
  static const selfCheck = MethodChannel('eel.bvr/selfcheck');
}
