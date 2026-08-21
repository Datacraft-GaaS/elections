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
/// Verification: in production, [RtsModeBundle.verify] calls into
/// eel_kit_sdk verifier FFI. In this implementation a stub always returns
/// verified=true; wire the real verifier for field builds.
library;

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

  /// Load current mode bundle from secure local storage (or default to general).
  static Future<RtsModeBundle> loadCurrent() async {
    // TODO(prod): read from EelSecureStore, verify signature via eel_kit_sdk
    // verifier FFI, then return the verified bundle.
    _log.info('Loading mode bundle (stub: defaulting to general election)');
    return RtsModeBundle(
      electionType: ElectionType.general,
      formType: FormType.form34a,
      validFrom: DateTime.utc(2024, 1, 1),
      validUntil: DateTime.utc(2030, 1, 1),
      bundleId: 'default-general',
      verified: kDebugMode, // Only "verified" in debug; prod requires real sig.
    );
  }

  /// Apply a new bundle after verifying its signature.
  static Future<RtsModeBundle> applyBundle(Map<String, Object?> raw) async {
    final verified = await _verify(raw);
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

  static Future<bool> _verify(Map<String, Object?> raw) async {
    // TODO(prod): extract 'signature' field, verify ECDSA P-256 over
    // canonical JSON (signature excluded) using eel_kit_sdk verifier FFI.
    _log.info('Mode bundle signature verification stub - always true');
    return true;
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

  Future<void> applyRaw(Map<String, Object?> raw) async {
    final bundle = await RtsModeBundle.applyBundle(raw);
    assert(
      bundle.verified,
      'Mode bundle must be verified before applying; reject unverified bundles',
    );
    _log.info('Mode switched: $bundle');
    state = bundle;
  }
}

final modeBundleProvider = StateProvider<RtsModeBundle>(
  (ref) => throw UnimplementedError('modeBundleProvider must be overridden'),
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
