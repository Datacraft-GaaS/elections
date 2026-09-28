/// Bootstrap FSM for party-agent app.
/// 
/// Implements the startup state machine per plan.md §2.4:
/// cold → integrityCheck → attestation → config → auth → roleRoute → ready

import 'package:eel_core_flutter/src/bootstrap/bootstrap_fsm.dart';

/// Bootstrap stage enum
enum PartyAgentBootstrapStage {
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

/// Immutable bootstrap state
class PartyAgentBootstrapState {
  final PartyAgentBootstrapStage stage;
  final String? supportCode;
  final String? detail;
  final int attempt;
  final bool tamperDetected;
  final String? attestationVerdict;
  final String? appConfigVersion;

  const PartyAgentBootstrapState({
    this.stage = PartyAgentBootstrapStage.cold,
    this.supportCode,
    this.detail,
    this.attempt = 0,
    this.tamperDetected = false,
    this.attestationVerdict,
    this.appConfigVersion,
  });

  PartyAgentBootstrapState copyWith({
    PartyAgentBootstrapStage? stage,
    String? supportCode,
    String? detail,
    int? attempt,
    bool? tamperDetected,
    String? attestationVerdict,
    String? appConfigVersion,
  }) {
    return PartyAgentBootstrapState(
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
  bool get isReady => stage == PartyAgentBootstrapStage.ready;
}

/// Step result types
sealed class PartyAgentStepResult {
  const PartyAgentStepResult();
}

class PartyAgentStepOk extends PartyAgentStepResult {
  final PartyAgentBootstrapState state;
  const PartyAgentStepOk(this.state);
}

class PartyAgentStepBlock extends PartyAgentStepResult {
  final PartyAgentBootstrapState state;
  final String reason;
  const PartyAgentStepBlock(this.state, this.reason);
}

/// Bootstrap FSM runner
class PartyAgentBootstrapFsm {
  final BootstrapFsm _inner;

  PartyAgentBootstrapFsm() : _inner = BootstrapFsm();

  Stream<PartyAgentStepResult> run() async* {
    var state = const PartyAgentBootstrapState();

    yield PartyAgentStepOk(state.copyWith(stage: PartyAgentBootstrapStage.cold));

    state = state.copyWith(stage: PartyAgentBootstrapStage.integrityCheck);
    yield PartyAgentStepOk(state);

    final integrityOk = await _inner.checkIntegrity();
    if (!integrityOk) {
      state = state.copyWith(
        stage: PartyAgentBootstrapStage.blockedRasp,
        tamperDetected: true,
        supportCode: _generateSupportCode(),
      );
      yield PartyAgentStepBlock(state, 'Device integrity check failed');
      return;
    }

    state = state.copyWith(stage: PartyAgentBootstrapStage.attestation);
    yield PartyAgentStepOk(state);

    final verdict = await _inner.verifyAttestation();
    if (verdict == null) {
      state = state.copyWith(
        stage: PartyAgentBootstrapStage.blockedAttest,
        supportCode: _generateSupportCode(),
      );
      yield PartyAgentStepBlock(state, 'Device attestation failed');
      return;
    }
    state = state.copyWith(attestationVerdict: verdict);

    state = state.copyWith(stage: PartyAgentBootstrapStage.config);
    yield PartyAgentStepOk(state);

    final configOk = await _inner.fetchConfig();
    if (!configOk) {
      state = state.copyWith(
        stage: PartyAgentBootstrapStage.blockedConfig,
        supportCode: _generateSupportCode(),
      );
      yield PartyAgentStepBlock(state, 'Config fetch failed');
      return;
    }

    state = state.copyWith(stage: PartyAgentBootstrapStage.auth);
    yield PartyAgentStepOk(state);

    state = state.copyWith(stage: PartyAgentBootstrapStage.roleRoute);
    yield PartyAgentStepOk(state);

    state = state.copyWith(stage: PartyAgentBootstrapStage.ready);
    yield PartyAgentStepOk(state);
  }

  String _generateSupportCode() {
    final timestamp = DateTime.now().millisecondsSinceEpoch;
    return 'PA-${timestamp.toRadixString(36).toUpperCase()}';
  }
}
