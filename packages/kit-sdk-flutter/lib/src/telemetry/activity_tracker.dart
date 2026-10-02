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

/// Flutter navigator observer + workflow-step marker API.
library;

import 'package:flutter/widgets.dart';

import 'signal_sink.dart';
import 'signal_types.dart';

/// Navigator observer that emits [ActivitySignal] on route changes.
class TelemetryNavigatorObserver extends NavigatorObserver {
  TelemetryNavigatorObserver({
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

  @override
  void didPush(Route<dynamic> route, Route<dynamic>? previousRoute) =>
      _emit('screen_push', route.settings.name);

  @override
  void didPop(Route<dynamic> route, Route<dynamic>? previousRoute) =>
      _emit('screen_pop', previousRoute?.settings.name);

  @override
  void didReplace({Route<dynamic>? newRoute, Route<dynamic>? oldRoute}) =>
      _emit('screen_replace', newRoute?.settings.name);

  void _emit(String kind, String? screenName) {
    final signal = ActivitySignal(
      deviceId: _deviceId,
      occurredAt: DateTime.now().toUtc(),
      monotonicCounter: _nextCounter(),
      traceId: _nextTraceId(),
      spanId: _nextSpanId(),
      correlationId: _nextTraceId(),
      prevHash: _currentPrevHash(),
      activityKind: kind,
      screenName: screenName,
    );
    _sink.accept(signal);
  }

  /// Emit a named workflow-step marker.
  void markWorkflowStep(String name) {
    final signal = ActivitySignal(
      deviceId: _deviceId,
      occurredAt: DateTime.now().toUtc(),
      monotonicCounter: _nextCounter(),
      traceId: _nextTraceId(),
      spanId: _nextSpanId(),
      correlationId: _nextTraceId(),
      prevHash: _currentPrevHash(),
      activityKind: 'workflow_step',
      workflowStep: name,
    );
    _sink.accept(signal);
  }
}
