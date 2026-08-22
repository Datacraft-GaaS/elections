import 'package:flutter_test/flutter_test.dart';
import 'package:eel_kit_sdk/telemetry.dart';

void main() {
  const sampling = AdaptiveSampling();

  group('AdaptiveSampling', () {
    test('100 Hz during active workflow with sufficient battery', () {
      final mode = sampling.mode(batteryPercent: 80, workflowActive: true);
      expect(mode, equals(SamplingMode.activeWorkflow));
      expect(sampling.hzFor(mode), equals(100));
    });

    test('10 Hz when idle with sufficient battery', () {
      final mode = sampling.mode(batteryPercent: 80, workflowActive: false);
      expect(mode, equals(SamplingMode.idle));
      expect(sampling.hzFor(mode), equals(10));
    });

    test('1 Hz when battery below 15 %', () {
      final mode = sampling.mode(batteryPercent: 14, workflowActive: true);
      expect(mode, equals(SamplingMode.lowBattery));
      expect(sampling.hzFor(mode), equals(1));
    });

    test('1 Hz exactly at 15 %', () {
      final mode = sampling.mode(batteryPercent: 15, workflowActive: true);
      expect(mode, equals(SamplingMode.lowBattery));
    });

    test('10 Hz when idle at 16 %', () {
      final mode = sampling.mode(batteryPercent: 16, workflowActive: false);
      expect(mode, equals(SamplingMode.idle));
    });

    test('interval for activeWorkflow is 10 ms (100 Hz)', () {
      expect(
        sampling.intervalFor(SamplingMode.activeWorkflow),
        equals(const Duration(milliseconds: 10)),
      );
    });

    test('interval for idle is 100 ms (10 Hz)', () {
      expect(
        sampling.intervalFor(SamplingMode.idle),
        equals(const Duration(milliseconds: 100)),
      );
    });

    test('interval for lowBattery is 1 s (1 Hz)', () {
      expect(
        sampling.intervalFor(SamplingMode.lowBattery),
        equals(const Duration(seconds: 1)),
      );
    });
  });
}
