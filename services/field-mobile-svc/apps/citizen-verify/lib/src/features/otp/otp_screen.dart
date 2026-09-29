/// OTP screen (M13 FR-C2) — Beautiful, ergonomic, accessible.
/// Now integrates with Real OtpService.sendOtp(phone) via telco-sms adapter.
/// M5 audit event emitted on every OTP attempt.

import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';
import 'package:eel_core_flutter/eel_core_flutter.dart';

import '../../core/services/otp_token.dart';

class OtpScreen extends ConsumerStatefulWidget {
  const OtpScreen({super.key});

  @override
  ConsumerState<OtpScreen> createState() => _OtpScreenState();
}

class _OtpScreenState extends ConsumerState<OtpScreen> {
  /// Every OTP attempt writes a 5-axis audit entry, so the service needs an
  /// AuditService; the screen owns one for its own lifetime.
  final OtpService _otp = OtpService(
    audit: AuditService('citizen-verify', 'citizen-verify'),
  );

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

    try {
      final response = await _otp.sendOtp(phone);
      setState(() => _codeSent = true);
      if (response.success && response.token != null) {
        // Keep it: RegisterService.lookup and fileObjection take this as a
        // bearer credential. It used to be dropped here, so the screens that
        // need it had no way to authorise the call.
        ref.read(otpTokenProvider.notifier).state = response.token;
      } else {
        setState(() => _error = response.error ?? 'Unknown error');
      }
    } catch (e) {
      setState(() => _error = e.toString());
    } finally {
      setState(() => _loading = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return EelScaffold(
      title: 'OTP Verification',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(24),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Enter your phone number to receive OTP', style: EelTextStyle.titleMedium),
          const SizedBox(height: 16),
          EelTextField(
            label: 'Phone Number',
            controller: _phone,
            keyboardType: TextInputType.phone,
            hintText: '+254701234567',
            enabled: !_loading,
          ),
          if (_error != null) ...[
            const SizedBox(height: 8),
            Text(_error!, style: const TextStyle(color: Colors.red)),
          ],
          const SizedBox(height: 16),
          EelButton(
            onPressed: _loading || _codeSent ? null : _sendOtp,
            label: _loading ? 'Sending...' : (_codeSent ? 'Resend OTP' : 'Send OTP'),
          ),
          if (_codeSent) ...[
            const SizedBox(height: 16),
            const Text('OTP sent! Please enter the code received.', style: EelTextStyle.bodySmall),
            const SizedBox(height: 12),
            EelTextField(
              label: 'OTP Code',
              controller: _code,
              hintText: '123456',
              enabled: !_loading,
            ),
          ],
        ],
      ),
    );
  }
}