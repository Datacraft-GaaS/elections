/// Bootstrap FSM for poll-clerk app.
/// 
/// Implements the startup state machine per plan.md §2.4:
/// cold → integrityCheck → attestation → config → auth → roleRoute → ready
/// 
/// States: cold, integrityCheck, attestation, config, auth, roleRoute, ready
/// Blocked states: blockedRasp, blockedAttest, blockedConfig, blockedAuth, blockedVersion

import 'package:eel_core_flutter/bootstrap/bootstrap_fsm.dart';

/// Bootstrap stage enum matching the FSM spec
enum PollClerkBootstrapStage {
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

/// Immutable bootstrap state for poll-clerk
class PollClerkBootstrapState {
  final PollClerkBootstrapStage stage;
  final String? supportCode;
  final String? detail;
  final int attempt;
  final bool tamperDetected;
  final String? attestationVerdict;
  final String? appConfigVersion;

  const PollClerkBootstrapState({
    this.stage = PollClerkBootstrapStage.cold,
    this.supportCode,
    this.detail,
    this.attempt = 0,
    this.tamperDetected = false,
    this.attestationVerdict,
    this.appConfigVersion,
  });

  PollClerkBootstrapState copyWith({
    PollClerkBootstrapStage? stage,
    String? supportCode,
    String? detail,
    int? attempt,
    bool? tamperDetected,
    String? attestationVerdict,
    String? appConfigVersion,
  }) {
    return PollClerkBootstrapState(
      stage: stage ?? this.stage,
      supportCode: supportCode ?? this.supportCode,
      detail: detail ?? this.detail,
      attempt: attempt ?? this.attempt,
      tamperDetected: tamperDetected ?? this.tamperDetected,
      attestationVerdict: attestationVerdict ?? this.attestationVerdict,
      appConfigVersion: appConfigVersion ?? this.appConfigVersion,
    );
  }

  bool get isBlocked => stage.name.startsWith('blocked');
  bool get isReady => stage == PollClerkBootstrapStage.ready;
}

/// Step result types for bootstrap FSM
sealed class PollClerkStepResult {
  const PollClerkStepResult();
}

class PollClerkStepOk extends PollClerkStepResult {
  final PollClerkBootstrapState state;
  const PollClerkStepOk(this.state);
}

class PollClerkStepBlock extends PollClerkStepResult {
  final PollClerkBootstrapState state;
  final String reason;
  const PollClerkStepBlock(this.state, this.reason);
}

/// Bootstrap FSM runner for poll-clerk
/// 
/// Returns a stream of state transitions and final result.
/// Usage:
/// ```dart
/// final fsm = PollClerkBootstrapFsm();
/// await for (final result in fsm.run()) {
///   // handle state changes and final result
/// }
/// ```
class PollClerkBootstrapFsm {
  final BootstrapFsm _inner;
  
  PollClerkBootstrapFsm() : _inner = BootstrapFsm();
  
  /// Run the bootstrap FSM and emit state transitions
  Stream<PollClerkStepResult> run() async* {
    var state = const PollClerkBootstrapState();
    
    // Stage 1: Cold start
    yield PollClerkStepOk(state.copyWith(stage: PollClerkBootstrapStage.cold));
    
    // Stage 2: Integrity check (RASP)
    state = state.copyWith(stage: PollClerkBootstrapStage.integrityCheck);
    yield PollClerkStepOk(state);
    
    final integrityOk = await _inner.checkIntegrity();
    if (!integrityOk) {
      state = state.copyWith(
        stage: PollClerkBootstrapStage.blockedRasp,
        tamperDetected: true,
        supportCode: _generateSupportCode(),
      );
      yield PollClerkStepBlock(state, 'Device integrity check failed');
      return;
    }
    
    // Stage 3: Attestation
    state = state.copyWith(stage: PollClerkBootstrapStage.attestation);
    yield PollClerkStepOk(state);
    
    final verdict = await _inner.verifyAttestation();
    if (verdict == null) {
      state = state.copyWith(
        stage: PollClerkBootstrapStage.blockedAttest,
        supportCode: _generateSupportCode(),
      );
      yield PollClerkStepBlock(state, 'Device attestation failed');
      return;
    }
    state = state.copyWith(attestationVerdict: verdict);
    
    // Stage 4: Config fetch
    state = state.copyWith(stage: PollClerkBootstrapStage.config);
    yield PollClerkStepOk(state);
    
    final configOk = await _inner.fetchConfig();
    if (!configOk) {
      state = state.copyWith(
        stage: PollClerkBootstrapStage.blockedConfig,
        supportCode: _generateSupportCode(),
      );
      yield PollClerkStepBlock(state, 'Config fetch failed');
      return;
    }
    
    // Stage 5: Auth
    state = state.copyWith(stage: PollClerkBootstrapStage.auth);
    yield PollClerkStepOk(state);
    
    // Auth is handled by session management - emit ready state
    // The actual auth flow is in session.dart
    state = state.copyWith(stage: PollClerkBootstrapStage.auth);
    yield PollClerkStepOk(state);
    
    // Stage 6: Role route
    state = state.copyWith(stage: PollClerkBootstrapStage.roleRoute);
    yield PollClerkStepOk(state);
    
    // Stage 7: Ready
    state = state.copyWith(stage: PollClerkBootstrapStage.ready);
    yield PollClerkStepOk(state);
  }
  
  String _generateSupportCode() {
    final timestamp = DateTime.now().millisecondsSinceEpoch;
    return 'PC-${timestamp.toRadixString(36).toUpperCase()}';
  }
}
