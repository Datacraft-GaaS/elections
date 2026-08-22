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
