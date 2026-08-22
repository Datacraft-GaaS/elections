/// Drop-event detector using accelerometer + gyroscope.
///
/// Emits a [DropSignal] when:
///   - Peak acceleration exceeds 3 g within a 200 ms window.
///   - OR free-fall (near-zero acceleration) followed by high-g spike.
library;

import 'dart:async';
import 'dart:math' as math;

import 'signal_sink.dart';
import 'signal_types.dart';

const double _dropThresholdG = 3.0;
const double _freeFallThresholdG = 0.3;
const int _windowMs = 200;
const double _gravityMs2 = 9.81;

/// Vector magnitude in g from raw m/s² values.
double _magnitude(double x, double y, double z) =>
    math.sqrt(x * x + y * y + z * z) / _gravityMs2;

class _Sample {
  _Sample(this.timestamp, this.magnitudeG);
  final DateTime timestamp;
  final double magnitudeG;
}

/// Detects drops from a stream of accelerometer readings.
class DropDetector {
  DropDetector({
    required String deviceId,
    required SignalSink sink,
    required String Function() nextTraceId,
    required String Function() nextSpanId,
    required int Function() nextCounter,
    required String Function() currentPrevHash,
  })  : _deviceId = deviceId,
        _sink = sink,
        _nextTraceId = nextTraceId,
        _nextSpanId = nextSpanId,
        _nextCounter = nextCounter,
        _currentPrevHash = currentPrevHash;

  final String _deviceId;
  final SignalSink _sink;
  final String Function() _nextTraceId;
  final String Function() _nextSpanId;
  final int Function() _nextCounter;
  final String Function() _currentPrevHash;

  final _window = <_Sample>[];
  bool _freeFall = false;

  /// Feed a new accelerometer sample (m/s²).
  Future<void> onAccelerometer(double x, double y, double z) async {
    final now = DateTime.now().toUtc();
    final mag = _magnitude(x, y, z);
    _window.add(_Sample(now, mag));

    // Prune samples older than the detection window.
    final cutoff = now.subtract(const Duration(milliseconds: _windowMs));
    _window.removeWhere((s) => s.timestamp.isBefore(cutoff));

    // Free-fall state: all recent samples near zero-g.
    final allFreeFall = _window.every((s) => s.magnitudeG < _freeFallThresholdG);
    if (allFreeFall) {
      _freeFall = true;
      return;
    }

    // Peak check.
    final peak = _window.fold<double>(0, (m, s) => math.max(m, s.magnitudeG));
    if (peak >= _dropThresholdG) {
      await _emitDrop(
        peakG: peak,
        freeFall: _freeFall,
        windowMs: _windowMs,
      );
      _freeFall = false;
      _window.clear();
    } else {
      _freeFall = false;
    }
  }

  Future<void> _emitDrop({
    required double peakG,
    required bool freeFall,
    required int windowMs,
  }) async {
    final signal = DropSignal(
      deviceId: _deviceId,
      occurredAt: DateTime.now().toUtc(),
      monotonicCounter: _nextCounter(),
      traceId: _nextTraceId(),
      spanId: _nextSpanId(),
      correlationId: _nextTraceId(),
      prevHash: _currentPrevHash(),
      peakAccelG: peakG,
      windowMs: windowMs,
      freeFallDetected: freeFall,
      severity: SignalSeverity.critical,
    );
    await _sink.accept(signal);
  }
}
