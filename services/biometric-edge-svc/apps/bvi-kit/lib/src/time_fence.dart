/// Time-fence FSM (EC-TIM-01).
///
/// A kit session may only open when:
///   - NTS sync has been attempted.
///   - Clock skew is <= [maxSkewSeconds] (5 min).
/// If skew is exceeded, the session is refused and the kit shows a
/// prominent error; no biometric capture proceeds.
library;

const int maxSkewSeconds = 300; // 5 minutes

enum TimeFenceState { unknown, syncing, withinFence, exceeded }

class TimeFence {
  final DateTime Function() _now;
  DateTime? _ntsTime;

  TimeFence({DateTime Function()? now}) : _now = now ?? DateTime.now;

  /// Record the NTS-acquired server time.
  void recordNtsTime(DateTime serverTime) {
    _ntsTime = serverTime;
  }

  /// Check whether the kit clock is within the allowed skew.
  TimeFenceState check() {
    if (_ntsTime == null) return TimeFenceState.unknown;
    final skew = _now().difference(_ntsTime!).abs().inSeconds;
    return skew <= maxSkewSeconds
        ? TimeFenceState.withinFence
        : TimeFenceState.exceeded;
  }

  /// Returns true if a session is allowed to open.
  bool get sessionAllowed => check() == TimeFenceState.withinFence;
}
