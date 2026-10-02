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

/// Field-support bootstrap FSM (M32). Roles: DATACRAFT_ENGINEER
/// Per plan.md M32: ticket management, kit diagnostics, shift reports
library;

import 'package:eel_core_flutter/eel_core_flutter.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

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

/// The three security gates bootstrap must pass, injected rather than assumed.
///
/// [integrityCheck] is the RASP / tamper check, [attestation] is the platform
/// attestation, and [fetchConfig] is the signed-config fetch. They are
/// constructor parameters with no default on purpose: a default that passed
/// would leave bootstrap always succeeding, which is the same as having no
/// gate at all, and a default that failed would block every launch. The app
/// has to state what it trusts.
typedef IntegrityCheck = Future<bool> Function();
typedef ConfigLoader = Future<bool> Function();

class FieldSupportBootstrapFsm {
  FieldSupportBootstrapFsm({
    required this.integrityCheck,
    required this.attestation,
    required this.fetchConfig,
  }) : _inner = BootstrapFsm(
          steps: {
            BootstrapStage.integrityCheck: (_) async =>
                await integrityCheck()
                    ? const StepOk()
                    : StepBlock(
                        stage: BootstrapStage.blockedRasp,
                        supportCode: _code('FS-RASP'),
                        detail: 'Integrity check failed',
                      ),
            BootstrapStage.attestation: (_) async =>
                _verdictIsAcceptable(await attestation.attest())
                    ? const StepOk()
                    : StepBlock(
                        stage: BootstrapStage.blockedAttest,
                        supportCode: _code('FS-ATTEST'),
                        detail: 'Attestation failed',
                      ),
            BootstrapStage.config: (_) async =>
                await fetchConfig()
                    ? const StepOk()
                    : StepBlock(
                        stage: BootstrapStage.blockedConfig,
                        supportCode: _code('FS-CONFIG'),
                        detail: 'Config fetch failed',
                      ),
          },
        );

  final IntegrityCheck integrityCheck;
  final EelAttestationClient attestation;
  final ConfigLoader fetchConfig;

  final BootstrapFsm _inner;

  /// A verdict is acceptable only if it is hardware-backed and current.
  ///
  /// A softwareFallback tier means no hardware attestation happened, and an
  /// expired verdict proves nothing about the device's present state. Both
  /// fail the gate. Note this makes EelAttestationClientStub always block,
  /// which is the intended direction: a stub must not stand in for hardware.
  static bool _verdictIsAcceptable(AttestationVerdict v) =>
      v.tier != AttestationTier.softwareFallback &&
      v.expiresAt.isAfter(DateTime.now());

  static String _code(String prefix) =>
      '$prefix-${DateTime.now().millisecondsSinceEpoch}';

  FieldSupportBootstrapState _toLocal(BootstrapState s) => FieldSupportBootstrapState(
        stage: FieldSupportBootstrapStage.values.byName(s.stage.name),
        supportCode: s.supportCode,
        detail: s.detail,
        attempt: s.attempt,
        tamperDetected: s.stage == BootstrapStage.blockedRasp,
        attestationVerdict:
            s.stage == BootstrapStage.blockedAttest ? 'failed' : null,
        appConfigVersion:
            s.stage == BootstrapStage.config ? 'fetched' : null,
      );

  /// Runs the shared FSM and reports every stage it passed through.
  ///
  /// The transitions are collected and replayed on return rather than yielded
  /// as they happen: a bootstrap runs to a terminal state before the UI can
  /// act on any of it, and a generator that yielded mid-run would leave the
  /// caller holding a half-finished sequence.
  Stream<FieldSupportStepResult> run() async* {
    final seen = <BootstrapState>[];
    void listener(BootstrapState s) => seen.add(s);

    _inner.addListener(listener);
    try {
      final terminal = await _inner.run();
      for (final s in seen) {
        yield FieldSupportStepOk(_toLocal(s));
      }
      if (terminal.stage == BootstrapStage.ready) {
        yield FieldSupportStepOk(_toLocal(terminal));
      } else {
        yield FieldSupportStepBlock(
          _toLocal(terminal),
          terminal.detail ?? 'Bootstrap blocked at ${terminal.stage.name}',
        );
      }
    } finally {
      _inner.removeListener(listener);
    }
  }
}
