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
