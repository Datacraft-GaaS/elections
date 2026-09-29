/// Startup FSM wiring (plan §2.4) for the CVR field-officer app.
///
/// Consumes eel_core_flutter's [BootstrapFsm] and maps each stage onto an
/// existing app service: attestation delegates to [AppIdentity] (kit-sdk),
/// the remaining stages are StepOk until their M1 backends land.
library;

import 'package:eel_core_flutter/eel_core_flutter.dart';

import 'services/app_identity.dart';

/// Ordered startup steps matching plan §2.4:
/// cold → integrityCheck → attestation → config → auth → roleRoute → ready.
class CvrBootstrap {
  CvrBootstrap._();

  static Future<BootstrapState> run() async => BootstrapFsm(steps: _steps).run();

  static final Map<BootstrapStage, BootstrapStep> _steps = {
    // NOT YET A GATE. The RASP probes (root/emulator/debugger/frida/xposed/
    // resign) arrive with the platform channels; until they exist this step
    // always passes. It is a StepOk, not a check, and should not be read as
    // one. Attestation below is real and does gate.
    BootstrapStage.integrityCheck: (_) async => const StepOk(),

    // Device attestation via kit-sdk (real client through AppIdentity).
    BootstrapStage.attestation: (state) async {
      try {
        await AppIdentity.instance.ensureAttested();
        return const StepOk(detail: 'attested');
      } on Exception catch (_) {
        return const StepBlock(
          stage: BootstrapStage.blockedAttest,
          supportCode: 'CVR-ATTEST-01',
          detail: 'Device attestation failed.',
        );
      }
    },

    // NOT YET A GATE: offline last-good config until the BFF lands.
    BootstrapStage.config: (_) async => const StepOk(),

    // NOT YET A GATE: OIDC + PKCE and biometric step-up land with M1 session.
    BootstrapStage.auth: (_) async => const StepOk(),

    // NOT YET A GATE: SpiceDB role from the session token lands with M1.
    BootstrapStage.roleRoute: (_) async => const StepOk(),
  };
}
