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
