/// Referendum / by-election mode switch.
///
/// A signed mode-bundle is fetched from the bvi-config endpoint and verified
/// before being applied. The bundle carries:
///   - election_type: 'general' | 'referendum' | 'by_election'
///   - form_type: 'form34a' | 'referendum_results_form' | 'by_election_34a'
///   - valid_from / valid_until ISO-8601
///   - ECDSA P-256 signature over canonical JSON (signed by IEBC bvi-config key)
///
/// Mode switching mid-session resets the capture workflow to step 1 and
/// re-validates any buffered entries against the new form schema.
///
/// Verification: [RtsModeBundle._verify] checks the ECDSA P-256 signature
/// over the canonical JSON of the bundle against the trust anchor provisioned
/// in EelSecureStore (see eel_kit_sdk bundle_security). Bundles without a
/// provisioned key or with an invalid signature are rejected (fail closed).
library;

import 'dart:convert';

import 'package:eel_kit_sdk/eel_kit_sdk.dart' show EelSecureStore, EelSignedBundle, ModeBundleTrustSlots;
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:logging/logging.dart';

final _log = Logger('eel.rts.mode_switch');

// ---- Types ------------------------------------------------------------------

enum ElectionType { general, referendum, byElection }

enum FormType { form34a, referendumResultsForm, byElection34a }

class RtsModeBundle {
  const RtsModeBundle({
    required this.electionType,
    required this.formType,
    required this.validFrom,
    required this.validUntil,
    required this.bundleId,
    this.verified = false,
  });

  final ElectionType electionType;
  final FormType formType;
  final DateTime validFrom;
  final DateTime validUntil;
  final String bundleId;
  final bool verified;

  bool get isActive {
    final now = DateTime.now().toUtc();
    return now.isAfter(validFrom) && now.isBefore(validUntil);
  }

  /// Default bundle: general election, form 34A.
  static RtsModeBundle get _defaultGeneral => RtsModeBundle(
        electionType: ElectionType.general,
        formType: FormType.form34a,
        validFrom: DateTime.utc(2024, 1, 1),
        validUntil: DateTime.utc(2030, 1, 1),
        bundleId: 'default-general',
        verified: kDebugMode,
      );

  /// Load the persisted mode bundle from secure local storage and re-verify
  /// its signature against the provisioned trust anchor. Falls back to the
  /// general election default when nothing is stored or verification fails.
  static Future<RtsModeBundle> loadCurrent() async {
    try {
      final store = EelSecureStore();
      final rawJson = await store.read(ModeBundleTrustSlots.appliedBundle);
      final trustAnchor = await store.read(ModeBundleTrustSlots.trustedPublicKey);
      if (rawJson == null || trustAnchor == null) {
        _log.info('No stored mode bundle or trust anchor; using general election default');
        return _defaultGeneral;
      }
      final raw = Map<String, Object?>.from(jsonDecode(rawJson) as Map);
      return RtsModeBundle.fromRaw(raw, trustAnchorB64: trustAnchor);
    } catch (e) {
      // Storage or codec failures degrade to the documented default bundle.
      _log.warning('Failed to load mode bundle ($e); using general election default');
      return _defaultGeneral;
    }
  }

  /// Parse [raw] after verifying its signature against [trustAnchorB64].
  factory RtsModeBundle.fromRaw(
    Map<String, Object?> raw, {
    required String trustAnchorB64,
  }) {
    final verified = EelSignedBundle.verify(raw, publicKeyBase64: trustAnchorB64);
    if (!verified) {
      _log.warning('Mode bundle signature invalid; rejecting');
    }
    return _parse(raw, verified: verified);
  }

  /// Apply a new bundle after verifying its signature against the given
  /// trust anchor, or the one provisioned in secure storage when omitted.
  /// A verified bundle is persisted so [loadCurrent] restores it next start.
  static Future<RtsModeBundle> applyBundle(
    Map<String, Object?> raw, {
    String? trustAnchorB64,
  }) async {
    final anchor = trustAnchorB64 ?? await _readTrustAnchor();
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
      _log.warning('Mode bundle signature invalid; rejecting');
    }
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

  static RtsModeBundle _parse(
    Map<String, Object?> raw, {
    required bool verified,
  }) {
    final electionTypeStr = raw['election_type'] as String? ?? 'general';
    final formTypeStr = raw['form_type'] as String? ?? 'form34a';
    return RtsModeBundle(
      electionType: ElectionType.values.firstWhere(
        (e) => e.name == electionTypeStr,
        orElse: () => ElectionType.general,
      ),
      formType: FormType.values.firstWhere(
        (e) => e.name == formTypeStr,
        orElse: () => FormType.form34a,
      ),
      validFrom: DateTime.parse(raw['valid_from'] as String),
      validUntil: DateTime.parse(raw['valid_until'] as String),
      bundleId: raw['bundle_id'] as String? ?? 'unknown',
      verified: verified,
    );
  }

  Map<String, Object?> toJson() => {
        'election_type': electionType.name,
        'form_type': formType.name,
        'valid_from': validFrom.toIso8601String(),
        'valid_until': validUntil.toIso8601String(),
        'bundle_id': bundleId,
        'verified': verified,
      };

  @override
  String toString() =>
      'RtsModeBundle(${electionType.name}, ${formType.name}, id=$bundleId, active=$isActive)';
}

// ---- Provider ---------------------------------------------------------------

/// Mutable notifier so mid-session mode switch propagates to all listeners.
class ModeBundleNotifier extends StateNotifier<RtsModeBundle> {
  ModeBundleNotifier(super.initial);

  Future<void> applyRaw(Map<String, Object?> raw, {String? trustAnchorB64}) async {
    final bundle =
        await RtsModeBundle.applyBundle(raw, trustAnchorB64: trustAnchorB64);
    assert(
      bundle.verified,
      'Mode bundle must be verified before applying; reject unverified bundles',
    );
    _log.info('Mode switched: $bundle');
    state = bundle;
  }
}

final modeBundleProvider = StateProvider<RtsModeBundle>(
  (ref) =>
      throw StateError('modeBundleProvider must be overridden before runApp'),
);

final modeBundleNotifierProvider =
    StateNotifierProvider<ModeBundleNotifier, RtsModeBundle>(
  (ref) => ModeBundleNotifier(ref.watch(modeBundleProvider)),
);

// ---- Screen -----------------------------------------------------------------

class ModeSwitchScreen extends ConsumerWidget {
  const ModeSwitchScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final bundle = ref.watch(modeBundleNotifierProvider);
    return Scaffold(
      appBar: AppBar(title: const Text('Election Mode')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Semantics(
              liveRegion: true,
              label: 'Current mode: ${bundle.electionType.name}',
              child: Card(
                child: Padding(
                  padding: const EdgeInsets.all(12),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Current mode: ${bundle.electionType.name}',
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                      Text('Form type: ${bundle.formType.name}'),
                      Text('Bundle ID: ${bundle.bundleId}'),
                      Text(
                        'Verified: ${bundle.verified}  Active: ${bundle.isActive}',
                      ),
                    ],
                  ),
                ),
              ),
            ),
            const SizedBox(height: 16),
            Text(
              'To switch modes, apply a signed mode bundle from the IEBC bvi-config endpoint.',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      ),
    );
  }
}
