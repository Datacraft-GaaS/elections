/// BVI Kit mode-switch handler for referendum and by-election modes.
///
/// Reads a signed mode-bundle from the bvi-config endpoint and applies it.
/// The bundle schema mirrors RTS mode_switch.dart with an additional
/// biometric_scope field for BVI-specific scope restriction.
///
/// Verification: in production, [BviModeBundle.verify] calls into
/// eel_kit_sdk verifier FFI. The stub here always returns verified=true;
/// wire the real verifier for field builds.
///
/// Mode fields:
///   - election_type: 'general' | 'referendum' | 'by_election'
///   - biometric_scope: 'constituency' | 'national' (by_election restricts to constituency)
///   - valid_from / valid_until ISO-8601
///   - bundle_id: opaque string
///   - signature: ECDSA P-256 over canonical JSON (signature key excluded)
library;

import 'package:flutter/foundation.dart';
import 'package:logging/logging.dart';

final _log = Logger('eel.bvi.mode_switch');

// ---- Types ------------------------------------------------------------------

enum BviElectionType { general, referendum, byElection }

/// BVI mode bundle -- parallel to RTS RtsModeBundle but with biometric_scope.
class BviModeBundle {
  const BviModeBundle({
    required this.electionType,
    required this.biometricScope,
    required this.validFrom,
    required this.validUntil,
    required this.bundleId,
    this.verified = false,
  });

  final BviElectionType electionType;

  /// 'national' for general/referendum; 'constituency' for by_election.
  /// Kit refuses to authenticate voters outside the scoped constituency
  /// when biometricScope == 'constituency'.
  final String biometricScope;

  final DateTime validFrom;
  final DateTime validUntil;
  final String bundleId;
  final bool verified;

  bool get isActive {
    final now = DateTime.now().toUtc();
    return now.isAfter(validFrom) && now.isBefore(validUntil);
  }

  /// Default bundle: general election, national scope.
  static BviModeBundle get defaultGeneral => BviModeBundle(
        electionType: BviElectionType.general,
        biometricScope: 'national',
        validFrom: DateTime.utc(2024, 1, 1),
        validUntil: DateTime.utc(2030, 1, 1),
        bundleId: 'default-general',
        verified: kDebugMode,
      );

  /// Load current bundle from secure local storage (or default to general).
  static Future<BviModeBundle> loadCurrent() async {
    // TODO(prod): read from EelSecureStore; verify ECDSA P-256 signature
    // via eel_kit_sdk verifier FFI before returning.
    _log.info('Loading BVI mode bundle (stub: defaulting to general)');
    return defaultGeneral;
  }

  /// Apply a new bundle from raw JSON after signature verification.
  /// Rejects bundles where verified == false (assertion enforced at caller).
  static Future<BviModeBundle> applyBundle(Map<String, Object?> raw) async {
    final verified = await _verify(raw);
    final typeStr = raw['election_type'] as String? ?? 'general';
    final scopeStr = raw['biometric_scope'] as String? ?? 'national';
    final bundle = BviModeBundle(
      electionType: BviElectionType.values.firstWhere(
        (e) => e.name == typeStr,
        orElse: () => BviElectionType.general,
      ),
      biometricScope: scopeStr,
      validFrom: DateTime.parse(raw['valid_from'] as String),
      validUntil: DateTime.parse(raw['valid_until'] as String),
      bundleId: raw['bundle_id'] as String? ?? 'unknown',
      verified: verified,
    );
    assert(
      bundle.verified,
      'BVI mode bundle must be verified before applying; unverified bundle rejected',
    );
    _log.info('BVI mode switched: ${bundle.electionType.name} scope=${bundle.biometricScope}');
    return bundle;
  }

  static Future<bool> _verify(Map<String, Object?> raw) async {
    // TODO(prod): extract 'signature' field; verify ECDSA P-256 over canonical
    // JSON (with signature key excluded) using eel_kit_sdk verifier FFI.
    _log.info('BVI mode bundle signature verification stub -- always true');
    return true;
  }

  /// Referendum-mode bundle with national scope.
  static BviModeBundle referendumBundle({
    required String bundleId,
    required DateTime validFrom,
    required DateTime validUntil,
  }) =>
      BviModeBundle(
        electionType: BviElectionType.referendum,
        biometricScope: 'national',
        validFrom: validFrom,
        validUntil: validUntil,
        bundleId: bundleId,
        verified: kDebugMode,
      );

  /// By-election bundle with constituency scope.
  /// Kits in constituency-scoped mode refuse biometric authentication for
  /// voters whose register entry is outside the constituency boundary.
  static BviModeBundle byElectionBundle({
    required String bundleId,
    required String constituencyId,
    required DateTime validFrom,
    required DateTime validUntil,
  }) =>
      BviModeBundle(
        electionType: BviElectionType.byElection,
        biometricScope: 'constituency:$constituencyId',
        validFrom: validFrom,
        validUntil: validUntil,
        bundleId: bundleId,
        verified: kDebugMode,
      );

  Map<String, Object?> toJson() => {
        'election_type': electionType.name,
        'biometric_scope': biometricScope,
        'valid_from': validFrom.toIso8601String(),
        'valid_until': validUntil.toIso8601String(),
        'bundle_id': bundleId,
        'verified': verified,
      };

  @override
  String toString() =>
      'BviModeBundle(${electionType.name}, scope=$biometricScope, id=$bundleId, active=$isActive)';
}
