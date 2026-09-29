/// Results Browser (M13 FR-C5/C6) — Beautiful, ergonomic, accessible.
import 'package:eel_design/eel_design.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import '../../core/services/results_service.dart';

/// Injected so a test can supply a Dio with a stubbed adapter.
final resultsServiceProvider = Provider<ResultsService>(
  (ref) => ResultsService(),
);

class ResultsScreen extends ConsumerStatefulWidget {
  const ResultsScreen({super.key});

  @override
  ConsumerState<ResultsScreen> createState() => _ResultsScreenState();
}

class _ResultsScreenState extends ConsumerState<ResultsScreen> {
  final _stationCode = TextEditingController();
  ResultsData? _results;
  bool? _proofVerified;
  bool _loading = false;
  String? _error;

  @override
  void dispose() {
    _stationCode.dispose();
    super.dispose();
  }

  /// Fetches station results and verifies the Merkle proof client-side.
  ///
  /// This screen used to render a hardcoded card reading "Station KE-NBI-001 …
  /// 87% • Verified" with a Load Results button whose handler was empty, so a
  /// citizen saw invented results presented as verified. Nothing is shown now
  /// until a real fetch succeeds, and what is shown comes from the response.
  Future<void> _load() async {
    final stationCode = _stationCode.text.trim();
    if (stationCode.isEmpty) {
      setState(() => _error = 'Enter a station code.');
      return;
    }

    setState(() {
      _loading = true;
      _error = null;
      _proofVerified = null;
    });

    final service = ref.read(resultsServiceProvider);
    final results = await service.fetchStationResults(stationCode: stationCode);

    bool? verified;
    final proof = results.proof;
    if (proof != null) {
      verified = await service.verifyProofClient(proof: proof);
    }

    if (!mounted) return;
    setState(() {
      _results = results;
      _proofVerified = verified;
      _loading = false;
      if (!results.ok) _error = results.error ?? 'Could not load results.';
    });
  }

  @override
  Widget build(BuildContext context) {
    return EelScaffold(
      title: 'Results',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Election Results — Verified',
              style: EelTextStyle.titleLarge),
          const SizedBox(height: 8),
          const EelStatusChip(
            label: 'National • County • Constituency • Station',
            severity: EelSeverity.success,
          ),
          const SizedBox(height: 16),
          EelTextField(
            label: 'Station code',
            hintText: 'KE-NBI-001',
            controller: _stationCode,
            enabled: !_loading,
          ),
          const SizedBox(height: 16),
          if (_error != null) ...[
            EelBanner(message: _error!, severity: EelSeverity.danger),
            const SizedBox(height: 16),
          ],
          if (_results != null && _results!.ok) ...[
            _card(context),
            const SizedBox(height: 16),
          ],
          EelButton(
            label: _loading ? 'Loading...' : 'Load Results',
            icon: Icons.refresh,
            fullWidth: true,
            onPressed: _loading ? null : _load,
          ),
        ],
      ),
    );
  }

  Widget _card(BuildContext context) {
    final r = _results!;
    final turnout = r.totalTurnout;
    final registered = r.totalVoters;
    final verified = _proofVerified;

    return EelResultCard(
      leading: Icon(
        verified == true ? Icons.verified : Icons.help_outline,
        color: verified == true
            ? EelColors.of(context).primary
            : EelColors.of(context).onSurfaceMuted,
        size: 32,
      ),
      title: Text(r.stationCode ?? 'Unknown station',
          style: EelTextStyle.titleMedium),
      subtitle: Text(
        <String?>[
          if (turnout != null && registered != null)
            'Turnout: $turnout / Expected: $registered',
          if (verified == true) 'Proof verified on device',
          if (verified == false) 'Proof could not be verified',
          if (verified == null) 'No proof supplied with this result',
        ].whereType<String>().join(' • '),
        style: EelTextStyle.bodyMedium,
      ),
    );
  }
}
