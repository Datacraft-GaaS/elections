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

/// Brackets user sessions with signed LoginLogoutSignals.
library;

import 'signal_sink.dart';
import 'signal_types.dart';

class SessionRecorder {
  SessionRecorder({
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

  DateTime? _loginAt;

  Future<void> recordLogin(String actorSvid, {String? biometricResult}) async {
    _loginAt = DateTime.now().toUtc();
    final signal = LoginLogoutSignal(
      deviceId: _deviceId,
      occurredAt: _loginAt!,
      monotonicCounter: _nextCounter(),
      traceId: _nextTraceId(),
      spanId: _nextSpanId(),
      correlationId: _nextTraceId(),
      prevHash: _currentPrevHash(),
      actorSvid: actorSvid,
      sessionEvent: SessionEvent.login,
      biometricResult: biometricResult,
    );
    await _sink.accept(signal);
  }

  Future<void> recordLogout(String actorSvid) async {
    final now = DateTime.now().toUtc();
    final durationMs = _loginAt != null ? now.difference(_loginAt!).inMilliseconds : null;
    final signal = LoginLogoutSignal(
      deviceId: _deviceId,
      occurredAt: now,
      monotonicCounter: _nextCounter(),
      traceId: _nextTraceId(),
      spanId: _nextSpanId(),
      correlationId: _nextTraceId(),
      prevHash: _currentPrevHash(),
      actorSvid: actorSvid,
      sessionEvent: SessionEvent.logout,
      sessionDurationMs: durationMs,
    );
    await _sink.accept(signal);
    _loginAt = null;
  }
}
