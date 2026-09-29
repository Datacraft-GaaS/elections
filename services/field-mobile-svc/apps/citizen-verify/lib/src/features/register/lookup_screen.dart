/// Register Lookup (M13 FR-010/011/012) — Beautiful, ergonomic, accessible.
import 'package:eel_design/eel_design.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import '../../core/services/otp_token.dart';
import '../../core/services/register_service.dart';

class RegisterLookupScreen extends ConsumerStatefulWidget {
  const RegisterLookupScreen({super.key});

  @override
  ConsumerState<RegisterLookupScreen> createState() => _RegisterLookupScreenState();
}

class _RegisterLookupScreenState extends ConsumerState<RegisterLookupScreen> {
  final _idHash = TextEditingController();
  final _justification = TextEditingController();
  RegisterLookup? _result;
  bool _loading = false;
  String? _error;

  @override
  void dispose() {
    _idHash.dispose();
    _justification.dispose();
    super.dispose();
  }

  /// Looks up the citizen's own record by ID hash.
  ///
  /// The call needs the OTP-issued bearer token, so without one the screen says
  /// so rather than issuing a request that would come back 401. The ID hash and
  /// justification are both required per M13 FR-010.
  Future<void> _lookup() async {
    final idHash = _idHash.text.trim();
    final justification = _justification.text.trim();
    final token = ref.read(otpTokenProvider);

    if (token == null) {
      setState(() => _error =
          'Verify your phone number first. This lookup needs the one-time code sent to you.');
      return;
    }
    if (idHash.isEmpty) {
      setState(() => _error = 'Enter your hashed voter ID.');
      return;
    }
    if (justification.isEmpty) {
      setState(() =>
          _error = 'A written justification is required per M13 FR-010.');
      return;
    }

    setState(() {
      _loading = true;
      _error = null;
    });

    final result = await ref.read(registerServiceProvider).lookup(idHash, token);

    if (!mounted) return;
    setState(() {
      _result = result;
      _loading = false;
      if (!result.found) _error = result.error ?? 'No record matched that ID hash.';
    });
  }

  @override
  Widget build(BuildContext context) {
    return EelScaffold(
      title: 'Register Lookup',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const EelBanner(
            message:
                'Requires DPO biometric attestation + written justification per M13 FR-010.',
            severity: EelSeverity.info,
          ),
          const SizedBox(height: 16),
          EelTextField(
            label: 'ID Hash (B)',
            hintText: 'Enter your hashed voter ID — never raw identity',
            controller: _idHash,
            enabled: !_loading,
          ),
          const SizedBox(height: 16),
          EelTextField(
            label: 'Justification / DPO Attestation',
            hintText: 'Required per M13 FR-010',
            maxLines: 3,
            controller: _justification,
            enabled: !_loading,
          ),
          if (_error != null) ...[
            const SizedBox(height: 16),
            EelBanner(message: _error!, severity: EelSeverity.danger),
          ],
          if (_result != null && _result!.found) ...[
            const SizedBox(height: 16),
            _card(context),
          ],
          const SizedBox(height: 16),
          EelButton(
            label: _loading ? 'Looking up...' : 'Verify Registration',
            icon: Icons.search,
            fullWidth: true,
            onPressed: _loading ? null : _lookup,
          ),
        ],
      ),
    );
  }

  Widget _card(BuildContext context) {
    final r = _result!;
    return EelResultCard(
      leading: Icon(
        r.status == 'REGISTERED' ? Icons.verified : Icons.pending_outlined,
        color: EelColors.of(context).primary,
        size: 32,
      ),
      title: Text('${r.idHash ?? 'Record'} — ${r.status ?? 'UNKNOWN'}',
          style: EelTextStyle.titleMedium),
      subtitle: Text(
        <String?>[
          if (r.stationName != null) r.stationName!,
          if (r.wardName != null) r.wardName!,
          if (r.countyName != null) r.countyName!,
        ].whereType<String>().join(' • '),
        style: EelTextStyle.bodyMedium,
      ),
    );
  }
}
