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
