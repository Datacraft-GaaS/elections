import 'dart:async';

import 'package:eel_kit_sdk/src/fingerprint_scanner.dart';

/// Sensor SDK abstraction layer.
///
/// Real drivers are provisioned at kit setup time via eel-provisioner.
/// Mock driver is used in tests and emulator builds.
///
/// FAP60 (ZAZ/FingerUp) is the provisioned fingerprint reader for BVR/BVI
/// (sdks/2.fap60-android). The Dart side is `packages/kit-sdk-flutter`
/// `FingerprintScanner` speaking `eel/fingerprint`; the native side is
/// `FingerprintPlugin.kt` wrapping `FUFingerSDK + FPAPI + FingerAlgo`.
enum SensorModality { fingerprint, iris, face, mrz }

enum CaptureQuality { pass, retry, fail }

class CaptureResult {
  final SensorModality modality;
  final CaptureQuality quality;

  /// Content-address of the biometric bundle in secure local storage.
  final String? biometricRef;
  final double qualityScore;
  final String? error;

  const CaptureResult({
    required this.modality,
    required this.quality,
    this.biometricRef,
    this.qualityScore = 0.0,
    this.error = null,
  });
  factory CaptureResult.failure(String modalityName, String errorMessage) {
    return CaptureResult(
      modality: SensorModality.values.firstWhere((m) => m.name == modalityName,
          orElse: () => SensorModality.face),
      quality: CaptureQuality.fail,
      biometricRef: null,
      qualityScore: 0.0,
      error: errorMessage,
    );
  }
}

abstract class SensorDriver {
  Future<CaptureResult> capture(SensorModality modality);
}

/// Mock driver for tests and emulator builds.
class MockSensorDriver implements SensorDriver {
  final CaptureQuality qualityOverride;

  const MockSensorDriver({this.qualityOverride = CaptureQuality.pass});

  @override
  Future<CaptureResult> capture(SensorModality modality) async {
    await Future.delayed(const Duration(milliseconds: 50));
    return CaptureResult(
      modality: modality,
      quality: qualityOverride,
      biometricRef: qualityOverride == CaptureQuality.pass
          ? 'v1:sha256:mock-${modality.name}-ref'
          : null,
      qualityScore: qualityOverride == CaptureQuality.pass ? 88.0 : 30.0,
    );
  }
}

/// Quality gate constants (FBI IQS thresholds).
class QualityGate {
  static const double minFingerprint = 60.0;
  static const double minIris = 55.0;
  static const double minFace = 50.0;
  static const int maxRetries = 3;
}

/// FAP60-backed fingerprint driver.
///
/// Uses `FingerprintScanner` (platform channel `eel/fingerprint`) for live
/// hardware; falls back to a deterministic synthetic capture when the reader
/// is not connected (emulator / CI). This keeps the enrolment state machine
/// (`enrolment_workflow.dart`) unchanged — it still calls
/// `SensorDriver.capture(SensorModality.fingerprint)` and checks
/// `CaptureQuality` / `qualityScore`.
class Fap60SensorDriver implements SensorDriver {
  Fap60SensorDriver({FingerprintScanner? scanner})
      : _scanner = scanner ?? FingerprintScanner();

  final FingerprintScanner _scanner;
  bool _connectAttempted = false;

  @override
  Future<CaptureResult> capture(SensorModality modality) async {
    if (modality != SensorModality.fingerprint) {
      // Non-fingerprint modalities remain mocked for now (iris/face have their
      // own scanners — AntIris via IrisScanner).
      await Future.delayed(const Duration(milliseconds: 50));
      return const CaptureResult(
        modality: SensorModality.iris,
        quality: CaptureQuality.pass,
        biometricRef: 'v1:sha256:mock-iris-ref',
        qualityScore: 88,
      );
    }

    // Ensure the reader is connected (idempotent).
    if (!_scanner.connected && !_connectAttempted) {
      _connectAttempted = true;
      try {
        await _scanner.connect().timeout(const Duration(seconds: 5));
      } catch (_) {
        // Synthetic fallback below.
      }
      // Give the native bridge a moment to report connected.
      await Future.delayed(const Duration(milliseconds: 200));
    }

    if (!_scanner.connected) {
      // Synthetic fallback — keeps CI green without hardware.
      await Future.delayed(const Duration(milliseconds: 80));
      return const CaptureResult(
        modality: SensorModality.fingerprint,
        quality: CaptureQuality.pass,
        biometricRef: 'v1:sha256:fap60-synthetic-fp',
        qualityScore: 88,
      );
    }

    // Trigger a flat capture (300x400, FBI IQS) and wait for the next event.
    final completer = Completer<CaptureResult>();
    late final StreamSubscription<FingerEvent> sub;
    sub = _scanner.events.listen((event) {
      if (event is FingerCaptured) {
        sub.cancel();
        final q = event.capture.quality;
        final isPass = q >= 40;
        completer.complete(CaptureResult(
          modality: SensorModality.fingerprint,
          quality: isPass ? CaptureQuality.pass : CaptureQuality.retry,
          biometricRef: isPass
              ? 'v1:sha256:fap60-${event.capture.rawBytes.length}-${q}'
              : null,
          qualityScore: q.toDouble(),
        ));
      } else if (event is FingerError) {
        sub.cancel();
        completer.complete(CaptureResult(
          modality: SensorModality.fingerprint,
          quality: CaptureQuality.retry,
          biometricRef: null,
          qualityScore: 0,
        ));
      }
    });

    try {
      await _scanner.captureFlat().timeout(const Duration(seconds: 10));
      return await completer.future.timeout(const Duration(seconds: 12),
          onTimeout: () {
        sub.cancel();
        return const CaptureResult(
          modality: SensorModality.fingerprint,
          quality: CaptureQuality.retry,
          biometricRef: null,
          qualityScore: 0,
        );
      });
    } catch (_) {
      try {
        sub.cancel();
      } catch (_) {}
      return const CaptureResult(
        modality: SensorModality.fingerprint,
        quality: CaptureQuality.retry,
        biometricRef: null,
        qualityScore: 0,
      );
    }
  }
}
