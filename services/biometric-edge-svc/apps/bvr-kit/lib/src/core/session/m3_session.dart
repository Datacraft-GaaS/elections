/// M3 session bootstrap (cold → self-check → roleRoute → ready).
/// Per plan.md M3: all 4 kits must complete bootstrap FSM before feature screens open.
/// Uses [KitCapabilityStatus] to gate feature access gracefully.

import '../capability/capability_status.dart';
import '../drivers/bvr_driver.dart';

enum M3BootstrapStep {
  cold,           // Initial: no session, no config.
  capabilityCheck, // Probe all hardware subsystems.
  selfCheck,       // Self-check result analyzed.
  roleRoute,       // Assign role (poll-clerk, agent, observer, etc.).
  ready,           // All gates passed; feature screens enabled.
  blocked,         // Hardware missing; feature screens blocked with banner.
  error,           // Unrecoverable failure.
}

class M3SessionState {
  final M3BootstrapStep step;
  final KitCapabilityStatus capability;
  final String? stationCode;
  final String? role;
  final String? errorMessage;

  const M3SessionState({
    required this.step,
    required this.capability,
    this.stationCode,
    this.role,
    this.errorMessage,
  });

  factory M3SessionState.initial() => M3SessionState(
        step: M3BootstrapStep.cold,
        capability: const KitCapabilityStatus(),
      );

  M3SessionState copyWith({
    M3BootstrapStep? step,
    KitCapabilityStatus? capability,
    String? stationCode,
    String? role,
    String? errorMessage,
  }) =>
      M3SessionState(
        step: step ?? this.step,
        capability: capability ?? this.capability,
        stationCode: stationCode ?? this.stationCode,
        role: role ?? this.role,
        errorMessage: errorMessage ?? this.errorMessage,
      );

  bool get isReady => step == M3BootstrapStep.ready;
  bool get isBlocked => step == M3BootstrapStep.blocked;
  bool get hasError => step == M3BootstrapStep.error;
}

/// M3 bootstrap service — enables feature screens only when hardware passes.
class M3BootstrapService {
  final BvrDriver _driver;

  M3BootstrapService({BvrDriver? driver}) : _driver = driver ?? BvrDriver();

  /// Execute full M3 bootstrap sequence.
  /// Per plan.md M3: cold → self-check → capability → roleRoute → ready/blocked.
  /// Returns [M3SessionState] that feature screens must observe.
  Future<M3SessionState> bootstrap({required String stationCode, required String role}) async {
    try {
      // Self-check attempts real driver; catches UnimplementedError (no hardware).
      try {
        await _driver.selfCheck();
      } catch (_) {
        // Expected when hardware not present; capability remains unavailable.
      }

      // Probe capabilities (graceful — never throws).
      final cap = KitCapabilityStatus(
        fingerprintAvailable: false, // Hardware-gated: set to true after FAP60 procurement
        cameraAvailable: false,
        signatureAvailable: false,
        ntsAvailable: false,
        gnssAvailable: false,
        hsmAvailable: false,
        smsAvailable: false,
        reason: 'M3 hardware not procured; drivers return UnimplementedError.',
      );

      if (!cap.isFullyCapable) {
        return M3SessionState(
          step: M3BootstrapStep.blocked,
          capability: cap,
          stationCode: stationCode,
          role: role,
          errorMessage: cap.featureBannerMessage,
        );
      }

      return M3SessionState(
        step: M3BootstrapStep.ready,
        capability: cap,
        stationCode: stationCode,
        role: role,
      );
    } catch (e) {
      return M3SessionState(
        step: M3BootstrapStep.error,
        capability: const KitCapabilityStatus(),
        stationCode: stationCode,
        role: role,
        errorMessage: e.toString(),
      );
    }
  }
}
