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

/// Periodic re-attestation heartbeat — emits [AttestationSignal] on schedule.
library;

import 'dart:async';

import 'signal_sink.dart';
import 'signal_types.dart';

class AttestationPulse {
  AttestationPulse({
    required String deviceId,
    required SignalSink sink,
    required String Function() nextTraceId,
    required String Function() nextSpanId,
    required int Function() nextCounter,
    required String Function() currentPrevHash,
    required Future<Map<String, Object?>> Function() attestProvider,
    Duration interval = const Duration(minutes: 15),
  })  : _deviceId = deviceId,
        _sink = sink,
        _nextTraceId = nextTraceId,
        _nextSpanId = nextSpanId,
        _nextCounter = nextCounter,
        _currentPrevHash = currentPrevHash,
        _attestProvider = attestProvider,
        _interval = interval;

  final String _deviceId;
  final SignalSink _sink;
  final String Function() _nextTraceId;
  final String Function() _nextSpanId;
  final int Function() _nextCounter;
  final String Function() _currentPrevHash;

  /// Returns a map with keys: status (String), kitCertValid (bool),
  /// nextCertRefreshUtc (String?), provider (String?).
  final Future<Map<String, Object?>> Function() _attestProvider;
  final Duration _interval;

  Timer? _timer;

  void start() {
    _pulse();
    _timer = Timer.periodic(_interval, (_) => _pulse());
  }

  void stop() {
    _timer?.cancel();
    _timer = null;
  }

  Future<void> _pulse() async {
    final info = await _attestProvider();
    final signal = AttestationSignal(
      deviceId: _deviceId,
      occurredAt: DateTime.now().toUtc(),
      monotonicCounter: _nextCounter(),
      traceId: _nextTraceId(),
      spanId: _nextSpanId(),
      correlationId: _nextTraceId(),
      prevHash: _currentPrevHash(),
      attestationStatus: info['status'] as String? ?? 'unknown',
      kitCertValid: info['kitCertValid'] as bool? ?? false,
      nextCertRefreshUtc: (info['nextCertRefreshUtc'] as String?) != null
          ? DateTime.tryParse(info['nextCertRefreshUtc'] as String)
          : null,
      attestationProvider: info['provider'] as String?,
    );
    await _sink.accept(signal);
  }
}
