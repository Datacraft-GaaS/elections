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
