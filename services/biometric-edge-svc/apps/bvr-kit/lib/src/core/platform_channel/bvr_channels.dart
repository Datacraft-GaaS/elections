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
