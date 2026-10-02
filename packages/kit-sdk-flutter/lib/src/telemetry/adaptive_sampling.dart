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

/// Battery-aware adaptive sampling rate controller.
///
/// Rules:
///   - Active workflow: 100 Hz sensor sampling.
///   - Idle (no active workflow): 10 Hz.
///   - Battery below 15 %: 1 Hz.
library;

/// Current sampling mode.
enum SamplingMode {
  /// 100 Hz — active workflow in progress.
  activeWorkflow,

  /// 10 Hz — app is open but no active workflow.
  idle,

  /// 1 Hz — battery below 15 %.
  lowBattery,
}

/// Determines [SamplingMode] from [batteryPercent] and [workflowActive].
class AdaptiveSampling {
  const AdaptiveSampling();

  static const int _lowBatteryThreshold = 15;

  SamplingMode mode({required int batteryPercent, required bool workflowActive}) {
    if (batteryPercent <= _lowBatteryThreshold) return SamplingMode.lowBattery;
    if (workflowActive) return SamplingMode.activeWorkflow;
    return SamplingMode.idle;
  }

  /// Returns the sampling interval for the given [mode].
  Duration intervalFor(SamplingMode mode) {
    return switch (mode) {
      SamplingMode.activeWorkflow => const Duration(milliseconds: 10), // 100 Hz
      SamplingMode.idle => const Duration(milliseconds: 100), // 10 Hz
      SamplingMode.lowBattery => const Duration(seconds: 1), // 1 Hz
    };
  }

  /// Returns the Hz value for the given [mode].
  int hzFor(SamplingMode mode) {
    return switch (mode) {
      SamplingMode.activeWorkflow => 100,
      SamplingMode.idle => 10,
      SamplingMode.lowBattery => 1,
    };
  }
}
