import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:flutter/foundation.dart';

/// Startup FSM (plan §2.4):
/// cold → integrityCheck → attestation → config → auth → roleRoute → ready
/// with blocked(reason) terminal states carrying a support code.
enum BootstrapStage {
  cold,
  integrityCheck,
  attestation,
  config,
  auth,
  roleRoute,
  ready,
  blockedRasp,
  blockedAttest,
  blockedConfig,
  blockedAuth,
  blockedVersion,
}

/// Immutable bootstrap state surfaced to the router and UI.
@immutable
class BootstrapState {
  const BootstrapState({
    required this.stage,
    this.supportCode,
    this.detail,
    this.attempt = 0,
  });

  final BootstrapStage stage;
  final String? supportCode;
  final String? detail;
  final int attempt;

  bool get isBlocked =>
      stage == BootstrapStage.blockedRasp ||
      stage == BootstrapStage.blockedAttest ||
      stage == BootstrapStage.blockedConfig ||
      stage == BootstrapStage.blockedAuth ||
      stage == BootstrapStage.blockedVersion;

  bool get isReady => stage == BootstrapStage.ready;

  BootstrapState copyWith({
    BootstrapStage? stage,
    String? supportCode,
    String? detail,
    int? attempt,
  }) =>
      BootstrapState(
        stage: stage ?? this.stage,
        supportCode: supportCode ?? this.supportCode,
        detail: detail ?? this.detail,
        attempt: attempt ?? this.attempt,
      );
}

/// Result of one FSM step: continue, or block with a support code.
sealed class StepResult {
  const StepResult();
}

class StepOk extends StepResult {
  const StepOk({this.detail});
  final String? detail;
}

class StepBlock extends StepResult {
  const StepBlock({required this.stage, required this.supportCode, this.detail});
  final BootstrapStage stage;
  final String supportCode;
  final String? detail;
}

/// A single FSM step. Implementations must be idempotent and offline-safe.
typedef BootstrapStep = Future<StepResult> Function(BootstrapState state);

/// Drives the startup FSM. Blocking steps hard-fail within the plan's ≤ 2 s
/// guidance; callers own timeouts per step.
class BootstrapFsm {
  BootstrapFsm({required this.steps});

  /// Ordered steps matching §2.4: integrityCheck, attestation, config, auth,
  /// roleRoute. `ready` is implicit after the last step succeeds.
  final Map<BootstrapStage, BootstrapStep> steps;

  BootstrapState _state = const BootstrapState(stage: BootstrapStage.cold);
  BootstrapState get state => _state;

  final List<void Function(BootstrapState)> _listeners = [];

  void addListener(void Function(BootstrapState) listener) =>
      _listeners.add(listener);

  void removeListener(void Function(BootstrapState) listener) =>
      _listeners.remove(listener);

  void _emit(BootstrapState next) {
    _state = next;
    for (final l in List.of(_listeners)) {
      l(next);
    }
  }

  /// Runs the FSM from [start] (default cold). Returns the terminal state.
  Future<BootstrapState> run({BootstrapState? from}) async {
    _state = from ?? const BootstrapState(stage: BootstrapStage.cold);
    final order = [
      BootstrapStage.integrityCheck,
      BootstrapStage.attestation,
      BootstrapStage.config,
      BootstrapStage.auth,
      BootstrapStage.roleRoute,
    ];
    for (final stage in order) {
      final step = steps[stage];
      if (step == null) continue;
      _emit(_state.copyWith(stage: stage));
      final result = await step(_state);
      switch (result) {
        case StepOk():
          _emit(
            _state.copyWith(
              stage: stage,
              detail: result.detail,
              attempt: 0,
            ),
          );
        case StepBlock():
          _emit(
            _state.copyWith(
              stage: result.stage,
              supportCode: result.supportCode,
              detail: result.detail,
              attempt: _state.attempt + 1,
            ),
          );
          return _state;
      }
    }
    _emit(_state.copyWith(stage: BootstrapStage.ready, attempt: 0));
    return _state;
  }
}