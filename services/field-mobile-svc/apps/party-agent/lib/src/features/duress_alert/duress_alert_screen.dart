/// Party-Agent Duress Alert (M30 FR-007) — Beautiful, ergonomic, accessible.
library;
import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

import '../../core/services/api_client.dart';
import '../auth/auth_provider.dart';
import 'duress_alert_service.dart';

/// The duress beacon, over the app's SPKI-pinned client.
///
/// Nothing constructed this before: DuressAlertService existed with no call
/// site, so the SEND DURESS ALERT button had nothing behind it.
final duressAlertServiceProvider = Provider<DuressAlertService>((ref) {
  final client = EelHttpClientFactory.create(
    baseUrl: const String.fromEnvironment(
      'EEL_BFF_BASE_URL',
      defaultValue: 'https://api.eel.local',
    ),
    attestation: EelAttestationClientStub(),
  );
  return DuressAlertService(client: client.client);
});

class DuressAlertScreen extends ConsumerStatefulWidget {
  const DuressAlertScreen({super.key});

  @override
  ConsumerState<DuressAlertScreen> createState() => _DuressAlertScreenState();
}

class _DuressAlertScreenState extends ConsumerState<DuressAlertScreen> {
  final _stationCode = TextEditingController();
  bool _sending = false;
  String? _error;
  String? _sentAt;

  @override
  void dispose() {
    _stationCode.dispose();
    super.dispose();
  }

  /// Sends the beacon.
  ///
  /// The service posts over the SPKI-pinned client and throws if none is
  /// configured, so a send that cannot be authenticated fails loudly here
  /// rather than appearing to have gone through.
  Future<void> _send() async {
    final agentId = ref.read(authProvider).agentId;
    final stationCode = _stationCode.text.trim();

    if (agentId == null || agentId.isEmpty) {
      setState(() =>
          _error = 'Sign in first: the beacon must name the agent it came from.');
      return;
    }
    if (stationCode.isEmpty) {
      setState(() => _error = 'Enter the station you are at.');
      return;
    }

    setState(() {
      _sending = true;
      _error = null;
    });

    try {
      await ref
          .read(duressAlertServiceProvider)
          .sendSilentAlert(agentId: agentId, stationCode: stationCode);
      if (!mounted) return;
      setState(() {
        _sending = false;
        _sentAt = DateTime.now().toIso8601String();
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _sending = false;
        _error = 'Duress alert was not delivered: $e';
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return EelScaffold(
      title: 'Duress Alert',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(24),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Send Duress Alert',
              style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 24),
          const EelBanner(
            message: 'This will notify security and authorities. Use only in genuine distress.',
            severity: EelSeverity.danger,
          ),
          const SizedBox(height: 24),
          EelResultCard(
            leading: Icon(Icons.warning, color: EelColors.of(context).primary, size: 32),
            title: const Text('Confirm Alert', style: EelTextStyle.titleMedium),
            subtitle: Text(
              _sentAt != null
                  ? 'Sent at $_sentAt. Stay where you are; help is on the way.'
                  : 'Press below to send duress signal.',
              style: EelTextStyle.bodyMedium,
            ),
          ),
          const SizedBox(height: 16),
          EelTextField(
            label: 'Station code',
            hintText: 'Station you are at',
            controller: _stationCode,
            enabled: !_sending,
          ),
          if (_error != null) ...[
            const SizedBox(height: 16),
            EelBanner(message: _error!, severity: EelSeverity.danger),
          ],
          const SizedBox(height: 16),
          EelButton(
            label: _sending ? 'Sending...' : 'SEND DURESS ALERT',
            icon: Icons.emergency,
            fullWidth: true,
            onPressed: _sending ? null : _send,
          ),
        ],
      ),
    );
  }
}
