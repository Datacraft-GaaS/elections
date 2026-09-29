/// Kit-L1 integration (M26/M28) — anomaly plugin integrated into bvi-kit / rts-kit pipelines.
/// Per plan.md M4: kit-l1 is a small integration task (2 ED) after M2/M3 complete.
library;

import '../src/anomaly_detector.dart';
import '../src/anomaly_result.dart';
import '../src/telemetry_point.dart';

/// Integration wrapper — consumes telemetry points from BVI (GNSS, fingerprint) and RTS (transmit events)
/// and produces anomaly results for the M4 pipeline.
class KitL1Pipeline {
  final EelKitL1AnomalyDetector _detector;
  KitL1Pipeline({EelKitL1AnomalyDetector? detector}) : _detector = detector ?? EelKitL1AnomalyDetector();

  /// Process telemetry point from BVI or RTS kit pipeline.
  /// Returns [AnomalyResult] with score + explanation.
  ///
  /// detect() fits itself on first use. Given a single point that fit is
  /// degenerate, so a caller scoring a live stream should call [processBatch]
  /// first to establish the model.
  Future<AnomalyResult> processPoint(TelemetryPoint point) async =>
      _detector.detect(<TelemetryPoint>[point]).first;

  /// Batch process multiple points (pipeline integration point).
  Future<List<AnomalyResult>> processBatch(List<TelemetryPoint> points) async =>
      _detector.detect(points);
}
