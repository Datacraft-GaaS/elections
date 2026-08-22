/// connectivity_plus adapter: emits NetworkSignal on every connectivity change.
library;

import 'dart:async';

import 'package:connectivity_plus/connectivity_plus.dart';

import 'signal_sink.dart';
import 'signal_types.dart';

class NetworkWatcher {
  NetworkWatcher({
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

  StreamSubscription<List<ConnectivityResult>>? _sub;

  void start() {
    _sub = Connectivity()
        .onConnectivityChanged
        .listen(_onConnectivityChanged);
  }

  void stop() {
    _sub?.cancel();
    _sub = null;
  }

  Future<void> _onConnectivityChanged(List<ConnectivityResult> results) async {
    final primary = results.isNotEmpty ? results.first : ConnectivityResult.none;
    final cls = _toClass(primary);
    final signal = NetworkSignal(
      deviceId: _deviceId,
      occurredAt: DateTime.now().toUtc(),
      monotonicCounter: _nextCounter(),
      traceId: _nextTraceId(),
      spanId: _nextSpanId(),
      correlationId: _nextTraceId(),
      prevHash: _currentPrevHash(),
      connectivityClass: cls,
    );
    await _sink.accept(signal);
  }

  ConnectivityClass _toClass(ConnectivityResult r) => switch (r) {
        ConnectivityResult.none => ConnectivityClass.offline,
        ConnectivityResult.wifi => ConnectivityClass.wifi,
        ConnectivityResult.mobile => ConnectivityClass.g4,
        ConnectivityResult.ethernet => ConnectivityClass.wifi,
        _ => ConnectivityClass.unknown,
      };
}
