/// Startup FSM wiring (plan §2.4) for the CVR field-officer app.
///
/// Consumes eel_core_flutter's [BootstrapFsm] and maps each stage onto an
/// existing app service: attestation delegates to [AppIdentity] (kit-sdk),
/// the remaining stages are StepOk until their M1 backends land.
library;

import 'package:eel_core_flutter/src/bootstrap/bootstrap_fsm.dart';

import 'services/app_identity.dart';

/// Ordered startup steps matching plan §2.4:
/// cold → integrityCheck → attestation → config → auth → roleRoute → ready.
class CvrBootstrap {
  CvrBootstrap._();

  static Future<BootstrapState> run() async => BootstrapFsm(steps: _steps).run();

  static final Map<BootstrapStage, BootstrapStep> _steps = {
    // RASP probes (root/emulator/debugger/frida/xposed/resign) land with the
    // platform channels; a clean probe set is StepOk.
    BootstrapStage.integrityCheck: (_) => const StepOk(),

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

    // Config fetch (M1 /config) is offline last-good until the BFF lands.
    BootstrapStage.config: (_) => const StepOk(),

    // OIDC code + PKCE and biometric step-up (M1 session).
    BootstrapStage.auth: (_) => const StepOk(),

    // SpiceDB-declared role from the app session token (M1).
    BootstrapStage.roleRoute: (_) => const StepOk(),
  };
}
