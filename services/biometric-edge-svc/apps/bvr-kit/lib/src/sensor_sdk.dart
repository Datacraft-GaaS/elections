/// Sensor SDK abstraction layer.
///
/// Real drivers are provisioned at kit setup time via eel-provisioner.
/// Mock driver is used in tests and emulator builds.
enum SensorModality { fingerprint, iris, face }

enum CaptureQuality { pass, retry, fail }

class CaptureResult {
  final SensorModality modality;
  final CaptureQuality quality;
  /// Content-address of the biometric bundle in secure local storage.
  final String? biometricRef;
  final double qualityScore;

  const CaptureResult({
    required this.modality,
    required this.quality,
    this.biometricRef,
    this.qualityScore = 0.0,
  });
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
