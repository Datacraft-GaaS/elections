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

/// sensors_plus adapter: 100 Hz raw sampling, 1 Hz downsample, drop + free-fall
/// detection, orientation-change detection.
library;

import 'dart:async';

import 'package:sensors_plus/sensors_plus.dart';

import 'adaptive_sampling.dart';
import 'drop_detector.dart';
import 'signal_sink.dart';
import 'signal_types.dart';

class SensorsService {
  SensorsService({
    required String deviceId,
    required SignalSink sink,
    required String Function() nextTraceId,
    required String Function() nextSpanId,
    required int Function() nextCounter,
    required String Function() currentPrevHash,
    AdaptiveSampling? sampling,
  })  : _deviceId = deviceId,
        _sink = sink,
        _nextTraceId = nextTraceId,
        _nextSpanId = nextSpanId,
        _nextCounter = nextCounter,
        _currentPrevHash = currentPrevHash,
        _sampling = sampling ?? const AdaptiveSampling(),
        _dropDetector = DropDetector(
          deviceId: deviceId,
          sink: sink,
          nextTraceId: nextTraceId,
          nextSpanId: nextSpanId,
          nextCounter: nextCounter,
          currentPrevHash: currentPrevHash,
        );

  final String _deviceId;
  final SignalSink _sink;
  final String Function() _nextTraceId;
  final String Function() _nextSpanId;
  final int Function() _nextCounter;
  final String Function() _currentPrevHash;
  final AdaptiveSampling _sampling;
  final DropDetector _dropDetector;

  StreamSubscription<UserAccelerometerEvent>? _accelSub;
  StreamSubscription<GyroscopeEvent>? _gyroSub;

  // 1 Hz downsampled state.
  DateTime? _lastDownsample;
  double _ax = 0, _ay = 0, _az = 0;
  double _gx = 0, _gy = 0, _gz = 0;

  SamplingMode _mode = SamplingMode.idle;

  void updateMode(SamplingMode mode) => _mode = mode;

  void start() {
    _accelSub = userAccelerometerEventStream(
      samplingPeriod: const Duration(milliseconds: 10), // 100 Hz
    ).listen(_onAccel);

    _gyroSub = gyroscopeEventStream(
      samplingPeriod: const Duration(milliseconds: 10),
    ).listen(_onGyro);
  }

  void stop() {
    _accelSub?.cancel();
    _gyroSub?.cancel();
    _accelSub = null;
    _gyroSub = null;
  }

  void _onAccel(UserAccelerometerEvent e) {
    _ax = e.x;
    _ay = e.y;
    _az = e.z;
    _dropDetector.onAccelerometer(e.x, e.y, e.z);
    _maybeDownsample();
  }

  void _onGyro(GyroscopeEvent e) {
    _gx = e.x;
    _gy = e.y;
    _gz = e.z;
  }

  void _maybeDownsample() {
    final interval = _sampling.intervalFor(_mode);
    // Raw 100 Hz passes through for activeWorkflow; 10 Hz / 1 Hz for others.
    final now = DateTime.now().toUtc();
    if (_lastDownsample != null && now.difference(_lastDownsample!) < interval) {
      return;
    }
    _lastDownsample = now;
    _emitMovement();
  }

  void _emitMovement() {
    final signal = MovementSignal(
      deviceId: _deviceId,
      occurredAt: DateTime.now().toUtc(),
      monotonicCounter: _nextCounter(),
      traceId: _nextTraceId(),
      spanId: _nextSpanId(),
      correlationId: _nextTraceId(),
      prevHash: _currentPrevHash(),
      accelX: _ax,
      accelY: _ay,
      accelZ: _az,
      gyroX: _gx,
      gyroY: _gy,
      gyroZ: _gz,
      isDownsampled: _mode != SamplingMode.activeWorkflow,
    );
    _sink.accept(signal);
  }
}
