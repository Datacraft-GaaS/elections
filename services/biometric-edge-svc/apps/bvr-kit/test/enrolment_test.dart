import 'package:flutter_test/flutter_test.dart';

import 'package:eel_bvr_kit/src/sensor_sdk.dart';

void main() {
  group('SensorDriver', () {
    test('mock driver returns pass result for fingerprint', () async {
      const driver = MockSensorDriver(qualityOverride: CaptureQuality.pass);
      final result = await driver.capture(SensorModality.fingerprint);
      expect(result.quality, CaptureQuality.pass);
      expect(result.biometricRef, isNotNull);
      expect(result.qualityScore, greaterThanOrEqualTo(QualityGate.minFingerprint));
    });

    test('mock driver returns fail result when overridden', () async {
      const driver = MockSensorDriver(qualityOverride: CaptureQuality.fail);
      final result = await driver.capture(SensorModality.fingerprint);
      expect(result.quality, CaptureQuality.fail);
      expect(result.biometricRef, isNull);
      expect(result.qualityScore, lessThan(QualityGate.minFingerprint));
    });

    test('mock driver supports face modality', () async {
      const driver = MockSensorDriver();
      final result = await driver.capture(SensorModality.face);
      expect(result.modality, SensorModality.face);
      expect(result.biometricRef, contains('face'));
    });

    test('quality gate constants are well-formed', () {
      expect(QualityGate.minFingerprint, greaterThan(0));
      expect(QualityGate.minIris, greaterThan(0));
      expect(QualityGate.minFace, greaterThan(0));
      expect(QualityGate.maxRetries, greaterThan(1));
    });
  });

  group('EnrolmentStep', () {
    test('all steps are distinct', () {
      final steps = EnrolmentStep.values;
      expect(steps.toSet().length, steps.length);
    });
  });
}

// Import enum for test without importing app widget.
enum EnrolmentStep {
  idle, capturingFingerprint, capturingIris, capturingFace,
  mrzScan, iprsVerify, signing, uploading, complete,
  degradedFinger, amputation, dualOfficer, error,
}
