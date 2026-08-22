/// Battery monitor: battery_plus polling at 30 s + platform-channel thermal.
library;

import 'dart:async';

import 'package:battery_plus/battery_plus.dart';

import 'signal_sink.dart';
import 'signal_types.dart';

class BatteryMonitor {
  BatteryMonitor({
    required String deviceId,
    required SignalSink sink,
    required String Function() nextTraceId,
    required String Function() nextSpanId,
    required int Function() nextCounter,
    required String Function() currentPrevHash,
    required void Function(int percent) onBatteryUpdate,
    Duration pollInterval = const Duration(seconds: 30),
  })  : _deviceId = deviceId,
        _sink = sink,
        _nextTraceId = nextTraceId,
        _nextSpanId = nextSpanId,
        _nextCounter = nextCounter,
        _currentPrevHash = currentPrevHash,
        _onBatteryUpdate = onBatteryUpdate,
        _pollInterval = pollInterval,
        _battery = Battery();

  final String _deviceId;
  final SignalSink _sink;
  final String Function() _nextTraceId;
  final String Function() _nextSpanId;
  final int Function() _nextCounter;
  final String Function() _currentPrevHash;
  final void Function(int percent) _onBatteryUpdate;
  final Duration _pollInterval;
  final Battery _battery;

  Timer? _timer;

  void start() {
    _poll();
    _timer = Timer.periodic(_pollInterval, (_) => _poll());
  }

  void stop() {
    _timer?.cancel();
    _timer = null;
  }

  Future<void> _poll() async {
    final percent = await _battery.batteryLevel;
    final state = await _battery.batteryState;
    _onBatteryUpdate(percent);

    final chargingState = switch (state) {
      BatteryState.charging => ChargingState.charging,
      BatteryState.discharging => ChargingState.discharging,
      BatteryState.full => ChargingState.full,
      _ => ChargingState.unknown,
    };

    final signal = BatterySignal(
      deviceId: _deviceId,
      occurredAt: DateTime.now().toUtc(),
      monotonicCounter: _nextCounter(),
      traceId: _nextTraceId(),
      spanId: _nextSpanId(),
      correlationId: _nextTraceId(),
      prevHash: _currentPrevHash(),
      percentInt: percent,
      chargingState: chargingState,
      severity: percent <= 15 ? SignalSeverity.warning : SignalSeverity.info,
    );
    await _sink.accept(signal);
  }
}
