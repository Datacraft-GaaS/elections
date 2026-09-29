/// Poll Clerk Login Screen (M28 FR-027) — Beautiful, ergonomic, accessible.
/// Uses eel_design: EelScaffold, EelTextField, EelButton, EelResultCard, EelColors, EelSecureScreen.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

import 'auth_provider.dart';

class LoginScreen extends ConsumerStatefulWidget {
  const LoginScreen({super.key});

  @override
  ConsumerState<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends ConsumerState<LoginScreen> {
  final _clerkId = TextEditingController();
  final _pin = TextEditingController();

  @override
  void dispose() {
    _clerkId.dispose();
    _pin.dispose();
    super.dispose();
  }

  /// Signs in through the real notifier, which hard-fails when device
  /// attestation does not pass (ADR-0021).
  Future<void> _signIn() async {
    final clerkId = _clerkId.text.trim();
    final pin = _pin.text;
    if (clerkId.isEmpty || pin.isEmpty) {
      _report('Enter both your clerk ID and password.');
      return;
    }
    await ref.read(authProvider.notifier).login(clerkId, pin);
  }

  void _report(String message) {
    if (!mounted) return;
    ScaffoldMessenger.of(context)
        .showSnackBar(SnackBar(content: Text(message)));
  }

  @override
  Widget build(BuildContext context) {
    final colors = EelColors.of(context);
    final auth = ref.watch(authProvider);
    final busy = auth.status == AuthStatus.loading;

    // Show a failed authentication where the operator is looking.
    ref.listen(authProvider, (previous, next) {
      final message = next.error;
      if (message != null && message != previous?.error) {
        _report(message);
      }
    });
    return EelScaffold(
      title: 'Login',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 32),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const SizedBox(height: 48),
          Center(
            child: EelSealStamp(size: 72, color: colors.primary),
          ),
          const SizedBox(height: 32),
          Text(
            'Welcome, Poll Clerk',
            style: EelTextStyle.displayMedium.copyWith(
              color: colors.onSurface,
            ),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 16),
          const Text(
            'Sign in to access station dashboard, voter lookup, and queue management.',
            style: EelTextStyle.bodyMedium,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 32),
          EelTextField(
            label: 'User ID',
            hintText: 'Enter your clerk ID',
            controller: _clerkId,
            enabled: !busy,
          ),
          const SizedBox(height: 16),
          EelTextField(
            label: 'Password',
            hintText: 'Enter your password',
            obscureText: true,
            controller: _pin,
            enabled: !busy,
          ),
          if (auth.error != null) ...[
            const SizedBox(height: 16),
            EelBanner(message: auth.error!, severity: EelSeverity.danger),
          ],
          const SizedBox(height: 24),
          EelButton(
            label: busy ? 'Signing in...' : 'Sign In',
            icon: Icons.login,
            fullWidth: true,
            onPressed: busy ? null : _signIn,
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Forgot Password',
            variant: EelButtonVariant.tertiary,
            fullWidth: true,
            // No recovery endpoint exists. Saying so beats a live-looking
            // button that does nothing.
            onPressed: busy
                ? null
                : () => _report(
                    'Contact your returning officer to have your clerk credentials reissued.',
                  ),
          ),
      // A Spacer is an Expanded, which needs a bounded height. This body sits
      // in EelScaffold's scrollable variant, so the height is unbounded and the
      // Spacer threw during layout. A fixed gap it is.
      const SizedBox(height: 32),
          Text(
            'eel Election Systems © 2026',
            style: EelTextStyle.bodySmall.copyWith(
              color: colors.onSurfaceMuted,
            ),
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }
}