/// Field-support bootstrap FSM (M32). Roles: DATACRAFT_ENGINEER
/// Per plan.md M32: ticket management, kit diagnostics, shift reports

import 'package:eel_core_flutter/bootstrap/bootstrap_fsm.dart';

enum FieldSupportBootstrapStage {
  cold, integrityCheck, attestation, config, auth, roleRoute, ready,
  blockedRasp, blockedAttest, blockedConfig, blockedAuth, blockedVersion,
}

class FieldSupportBootstrapState {
  final FieldSupportBootstrapStage stage;
  final String? supportCode; final String? detail;
  final int attempt; final bool tamperDetected;
  final String? attestationVerdict; final String? appConfigVersion;
  const FieldSupportBootstrapState({
    this.stage = FieldSupportBootstrapStage.cold, this.supportCode, this.detail,
    this.attempt = 0, this.tamperDetected = false,
    this.attestationVerdict, this.appConfigVersion,
  });
  FieldSupportBootstrapState copyWith({
    FieldSupportBootstrapStage? stage, String? supportCode, String? detail,
    int? attempt, bool? tamperDetected, String? attestationVerdict, String? appConfigVersion,
  }) => FieldSupportBootstrapState(
    stage: stage ?? this.stage, supportCode: supportCode ?? this.supportCode,
    detail: detail ?? this.detail, attempt: attempt ?? this.attempt,
    tamperDetected: tamperDetected ?? this.tamperDetected,
    attestationVerdict: attestationVerdict ?? this.attestationVerdict,
    appConfigVersion: appConfigVersion ?? this.appConfigVersion,
  );
  bool get isBlocked => stage.name.startsWith('blocked');
  bool get isReady => stage == FieldSupportBootstrapStage.ready;
}

sealed class FieldSupportStepResult { const FieldSupportStepResult(); }
class FieldSupportStepOk extends FieldSupportStepResult {
  final FieldSupportBootstrapState state;
  const FieldSupportStepOk(this.state);
}
class FieldSupportStepBlock extends FieldSupportStepResult {
  final FieldSupportBootstrapState state;
  final String reason;
  const FieldSupportStepBlock(this.state, this.reason);
}

class FieldSupportBootstrapFsm {
  final BootstrapFsm _inner;
  FieldSupportBootstrapFsm() : _inner = BootstrapFsm();

  Stream<FieldSupportStepResult> run() async* {
    var s = const FieldSupportBootstrapState();
    yield FieldSupportStepOk(s.copyWith(stage: FieldSupportBootstrapStage.cold));

    s = s.copyWith(stage: FieldSupportBootstrapStage.integrityCheck);
    yield FieldSupportStepOk(s);
    if (!(await _inner.checkIntegrity())) {
      s = s.copyWith(stage: FieldSupportBootstrapStage.blockedRasp, tamperDetected: true, supportCode: 'FS-${DateTime.now().millisecondsSinceEpoch}');
      yield FieldSupportStepBlock(s, 'Integrity check failed');
      return;
    }

    s = s.copyWith(stage: FieldSupportBootstrapStage.attestation);
    yield FieldSupportStepOk(s);
    if ((await _inner.verifyAttestation()) == null) {
      s = s.copyWith(stage: FieldSupportBootstrapStage.blockedAttest, supportCode: 'FS-${DateTime.now().millisecondsSinceEpoch}');
      yield FieldSupportStepBlock(s, 'Attestation failed');
      return;
    }

    s = s.copyWith(stage: FieldSupportBootstrapStage.config);
    yield FieldSupportStepOk(s);
    if (!(await _inner.fetchConfig())) {
      s = s.copyWith(stage: FieldSupportBootstrapStage.blockedConfig, supportCode: 'FS-${DateTime.now().millisecondsSinceEpoch}');
      yield FieldSupportStepBlock(s, 'Config fetch failed');
      return;
    }

    s = s.copyWith(stage: FieldSupportBootstrapStage.auth);
    yield FieldSupportStepOk(s);
    s = s.copyWith(stage: FieldSupportBootstrapStage.roleRoute);
    yield FieldSupportStepOk(s);
    s = s.copyWith(stage: FieldSupportBootstrapStage.ready);
    yield FieldSupportStepOk(s);
  }
}
