enum TimeConfidence { high, medium, low }

enum TimeSource { nts, gnss, monotonic }

class TimeFenceState {
  const TimeFenceState({
    required this.confidence,
    required this.source,
    required this.wallMs,
  });

  final TimeConfidence confidence;
  final TimeSource source;
  final int wallMs;
}

/// FSM for kit time authority (NTS + GNSS + monotonic fallback).
///
/// Refuses session opens with [TimeConfidence.low] unless [allowLow] is set.
class EelTimeFence {
  EelTimeFence({this.allowLow = false});

  final bool allowLow;

  TimeFenceState _state = TimeFenceState(
    confidence: TimeConfidence.low,
    source: TimeSource.monotonic,
    wallMs: DateTime.now().millisecondsSinceEpoch,
  );

  TimeFenceState get state => _state;

  void update(TimeFenceState newState) => _state = newState;

  bool canOpenSession() =>
      allowLow || _state.confidence != TimeConfidence.low;

  void assertCanOpenSession() {
    if (!canOpenSession()) {
      throw StateError('time_confidence: low - session open refused');
    }
  }
}
