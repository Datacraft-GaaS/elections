/// OTP login screen (M13 FR-C2) — Beautiful, ergonomic, accessible.
/// Uses eel_design: EelScaffold, EelTextField, EelButton, EelResultCard, EelColors.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverprovider.dart';
import 'package:eel_design/eel_design.dart';
import 'package:eel_design/widgets/eel_text_field.dart';

class OtpScreen extends ConsumerState StatefulWidget {
  const OtpScreen({super.key});
  @override
  ConsumerState<OtpScreen> createState() => _OtpScreenState();
}

class _OtpScreenState extends ConsumerState<OtpScreen> {
  final _phone = TextEditingController();
  final _code = TextEditingController();
  bool _codeSent = false;
  bool _loading = false;
  String? _error;

  Future<void> _sendOtp() async {
    final phone = _phone.text.trim();
    if (phone.isEmpty || phone.length < 10) {
      setState(() => _error = 'Enter a valid phone number');
      return;
    }
    setState(() { _loading = true; _error = null; });
    // Real: OtpService.sendOtp(phone)
    await Future.delayed(const Duration(seconds: 1));
    if (!mounted) return;
    setState(() { _loading = false; _codeSent = true; _error = null; });
  }

  Future<void> _verifyOtp() async {
    final code = _code.text.trim();
    if (code.length != 6) {
      setState(() => _error = 'OTP must be 6 digits');
      return;
    }
    setState(() { _loading = true; _error = null; });
    await Future.delayed(const Duration(seconds: 1));
    if (!mounted) return;
    setState(() { _loading = false; });
    Navigator.of(context).pushReplacementNamed('/register-lookup');
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final colors = EelColors.of(context);
    return EelScaffold(
      title: 'Verify Your Phone',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 32),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const SizedBox(height: 32),
          Center(
            child: EelSealStamp(size: 64, color: colors.primary),
          ),
          const SizedBox(height: 24),
          Text(
            'Enter your phone number',
            style: EelTextStyle.titleLarge,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 8),
          Text(
            'Used only to send a one-time password. Never stored.',
            style: EelTextStyle.bodyMedium,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 24),
          EelTextField(
            controller: _phone,
            label: 'Phone number',
            prefixText: '+',
            hint: '+254700000000',
            enabled: !_codeSent,
          ),
          if (_codeSent) ...[
            const SizedBox(height: 16),
            EelTextField(
              controller: _code,
              label: 'Enter OTP',
              hint: '000000',
              maxLength: 6,
              obscuringCharacter: '•',
            ),
          ],
          if (_error != null) ...[
            const SizedBox(height: 8),
            EelBanner(message: _error!, severity: 'critical'),
          ],
          const SizedBox(height: 16),
          if (_loading)
            const Center(child: EelLoadingState())
          else if (!_codeSent)
            EelButton(
              label: 'Send OTP',
              icon: Icons.send,
              fullWidth: true,
              onPressed: _sendOtp,
            )
          else
            EelButton(
              label: 'Verify OTP',
              icon: Icons.verified,
              fullWidth: true,
              onPressed: _verifyOtp,
            ),
        ],
      ),
    );
  }
}