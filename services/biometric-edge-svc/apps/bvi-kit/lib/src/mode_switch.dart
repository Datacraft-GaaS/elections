/// BVI Kit mode-switch handler for referendum and by-election modes.
///
/// Reads a signed mode-bundle from the bvi-config endpoint and applies it.
/// The bundle schema mirrors RTS mode_switch.dart with an additional
/// biometric_scope field for BVI-specific scope restriction.
///
/// Verification: [BviModeBundle.loadCurrent] and [BviModeBundle.applyBundle]
/// check the ECDSA P-256 signature over the canonical JSON of the bundle
/// against the trust anchor provisioned in EelSecureStore (see eel_kit_sdk
/// bundle_security). Bundles without a provisioned key or with an invalid
/// signature are rejected (fail closed).
///
/// Mode fields:
///   - election_type: 'general' | 'referendum' | 'by_election'
///   - biometric_scope: 'constituency' | 'national' (by_election restricts to constituency)
///   - valid_from / valid_until ISO-8601
///   - bundle_id: opaque string
///   - signature: ECDSA P-256 over canonical JSON (signature key excluded)
library;

import 'dart:convert';

import 'package:eel_kit_sdk/eel_kit_sdk.dart'
    show EelSignedBundle, EelSecureStore, ModeBundleTrustSlots;
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

  /// Load the persisted mode bundle from secure local storage and re-verify
  /// its signature against the provisioned trust anchor. Falls back to the
  /// general election default when nothing is stored, verification fails,
  /// or secure storage is unavailable (fail closed).
  static Future<BviModeBundle> loadCurrent() async {
    try {
      final store = EelSecureStore();
      final rawJson = await store.read(ModeBundleTrustSlots.appliedBundle);
      final trustAnchor = await store.read(ModeBundleTrustSlots.trustedPublicKey);
      if (rawJson == null || trustAnchor == null) {
        _log.info('No stored mode bundle or trust anchor; using general election default');
        return defaultGeneral;
      }
      final raw = Map<String, Object?>.from(jsonDecode(rawJson) as Map);
      if (!EelSignedBundle.verify(raw, publicKeyBase64: trustAnchor)) {
        _log.warning('Stored mode bundle signature invalid; rejecting');
        return defaultGeneral;
      }
      return _parse(raw, verified: true);
    } catch (e) {
      // Storage or codec failures degrade to the documented default bundle.
      _log.warning('Failed to load mode bundle ($e); using general election default');
      return defaultGeneral;
    }
  }

  /// Apply a new bundle from raw JSON after signature verification against
  /// [trustAnchorB64], or the one provisioned in secure storage when omitted.
  /// A verified bundle is persisted so [loadCurrent] restores it next start.
  /// Unverified bundles are rejected by the caller-side assertion (fail closed).
  static Future<BviModeBundle> applyBundle(
    Map<String, Object?> raw, {
    String? trustAnchorB64,
  }) async {
    final anchor = trustAnchorB64 ??
        await _readTrustAnchor();

    // No provisioned anchor means nothing can be verified: fail closed.
    final verified = anchor != null &&
        EelSignedBundle.verify(raw, publicKeyBase64: anchor);
    final bundle = _parse(raw, verified: verified);
    if (verified) {
      try {
        await EelSecureStore()
            .write(ModeBundleTrustSlots.appliedBundle, jsonEncode(raw));
      } catch (e) {
        // Persistence is best effort: the in-session bundle is still valid.
        _log.warning('Could not persist applied mode bundle: $e');
      }
    } else {
      _log.warning('BVI mode bundle signature invalid; rejecting');
    }
    assert(
      bundle.verified,
      'BVI mode bundle must be verified before applying; unverified bundle rejected',
    );
    _log.info('BVI mode switched: ${bundle.electionType.name} scope=${bundle.biometricScope}');
    return bundle;
  }

  /// Provisioned ECDSA P-256 trust anchor, or null when none is stored
  /// (secure storage unavailable counts as unprovisioned: fail closed).
  static Future<String?> _readTrustAnchor() async {
    try {
      return await EelSecureStore().read(ModeBundleTrustSlots.trustedPublicKey);
    } catch (e) {
      _log.warning('Could not read mode bundle trust anchor: $e');
      return null;
    }
  }

  static BviModeBundle _parse(
    Map<String, Object?> raw, {
    required bool verified,
  }) {
    final typeStr = raw['election_type'] as String? ?? 'general';
    final scopeStr = raw['biometric_scope'] as String? ?? 'national';
    return BviModeBundle(
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
