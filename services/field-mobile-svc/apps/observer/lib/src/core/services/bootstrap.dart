/// Observer bootstrap FSM (M34). Roles: OBSERVER / OBSERVER_CHIEF
/// Per plan.md M34: station results access, observation ingest, accreditation

import 'package:eel_core_flutter/src/bootstrap/bootstrap_fsm.dart';

enum ObserverBootstrapStage {
  cold, integrityCheck, attestation, config, auth, roleRoute, ready,
  blockedRasp, blockedAttest, blockedConfig, blockedAuth, blockedVersion,
}

class ObserverBootstrapState {
  final ObserverBootstrapStage stage;
  final String? supportCode; final String? detail;
  final int attempt; final bool tamperDetected;
  final String? attestationVerdict; final String? appConfigVersion;
  const ObserverBootstrapState({
    this.stage = ObserverBootstrapStage.cold,
    this.supportCode, this.detail, this.attempt = 0,
    this.tamperDetected = false, this.attestationVerdict, this.appConfigVersion,
  });
  ObserverBootstrapState copyWith({
    ObserverBootstrapStage? stage, String? supportCode, String? detail,
    int? attempt, bool? tamperDetected, String? attestationVerdict, String? appConfigVersion,
  }) => ObserverBootstrapState(
    stage: stage ?? this.stage,
    supportCode: supportCode ?? this.supportCode,
    detail: detail ?? this.detail,
    attempt: attempt ?? this.attempt,
    tamperDetected: tamperDetected ?? this.tamperDetected,
    attestationVerdict: attestationVerdict ?? this.attestationVerdict,
    appConfigVersion: appConfigVersion ?? this.appConfigVersion,
  );
  bool get isBlocked => stage.name.startsWith('blocked');
  bool get isReady => stage == ObserverBootstrapStage.ready;
}

sealed class ObserverStepResult {
  const ObserverStepResult();
}

class ObserverStepOk extends ObserverStepResult {
  final ObserverBootstrapState state;
  const ObserverStepOk(this.state);
}

class ObserverStepBlock extends ObserverStepResult {
  final ObserverBootstrapState state;
  final String reason;
  const ObserverStepBlock(this.state, this.reason);
}

class ObserverBootstrapFsm {
  final BootstrapFsm _inner;
  ObserverBootstrapFsm() : _inner = BootstrapFsm();

  Stream<ObserverStepResult> run() async* {
    var state = const ObserverBootstrapState();
    yield ObserverStepOk(state.copyWith(stage: ObserverBootstrapStage.cold));

    state = state.copyWith(stage: ObserverBootstrapStage.integrityCheck);
    yield ObserverStepOk(state);
    if (!(await _inner.checkIntegrity())) {
      state = state.copyWith(stage: ObserverBootstrapStage.blockedRasp, tamperDetected: true, supportCode: 'OBS-${DateTime.now().millisecondsSinceEpoch}');
      yield ObserverStepBlock(state, 'Integrity check failed');
      return;
    }

    state = state.copyWith(stage: ObserverBootstrapStage.attestation);
    yield ObserverStepOk(state);
    if ((await _inner.verifyAttestation()) == null) {
      state = state.copyWith(stage: ObserverBootstrapStage.blockedAttest, supportCode: 'OBS-${DateTime.now().millisecondsSinceEpoch}');
      yield ObserverStepBlock(state, 'Attestation failed');
      return;
    }

    state = state.copyWith(stage: ObserverBootstrapStage.config);
    yield ObserverStepOk(state);
    if (!(await _inner.fetchConfig())) {
      state = state.copyWith(stage: ObserverBootstrapStage.blockedConfig, supportCode: 'OBS-${DateTime.now().millisecondsSinceEpoch}');
      yield ObserverStepBlock(state, 'Config fetch failed');
      return;
    }

    state = state.copyWith(stage: ObserverBootstrapStage.auth);
    yield ObserverStepOk(state);

    state = state.copyWith(stage: ObserverBootstrapStage.roleRoute);
    yield ObserverStepOk(state);

    state = state.copyWith(stage: ObserverBootstrapStage.ready);
    yield ObserverStepOk(state);
  }
}
