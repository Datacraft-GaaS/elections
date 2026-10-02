// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Party-Agent Login — Beautiful, ergonomic, accessible.
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
  final _agentId = TextEditingController();
  final _pin = TextEditingController();

  @override
  void dispose() {
    _agentId.dispose();
    _pin.dispose();
    super.dispose();
  }

  /// Signs in through the real notifier.
  ///
  /// The screen used to carry a `Sign In` button whose handler was an empty
  /// closure, so the button looked live and did nothing. AuthNotifier.login is
  /// the actual gate, and it hard-fails when device attestation does not pass
  /// (ADR-0021), so wiring to it is the difference between a login screen and
  /// a picture of one.
  Future<void> _signIn() async {
    final agentId = _agentId.text.trim();
    final pin = _pin.text;
    if (agentId.isEmpty || pin.isEmpty) {
      _report('Enter both your agent ID and PIN.');
      return;
    }
    await ref.read(authProvider.notifier).login(agentId, pin);
  }

  void _report(String message) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message)),
    );
  }

  @override
  Widget build(BuildContext context) {
    final colors = EelColors.of(context);
    final auth = ref.watch(authProvider);
    final busy = auth.status == AuthStatus.loading;
    final error = auth.error;

    // Surface an authentication failure in place, where the operator is looking,
    // rather than only in a transient snackbar.
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
            child: EelSealStamp(size: 64, color: colors.primary),
          ),
          const SizedBox(height: 32),
          const Text('Welcome, Party Agent', style: EelTextStyle.displayMedium, textAlign: TextAlign.center),
          const SizedBox(height: 16),
          const Text('Sign in to access check-in, forms, and countersign tools.',
              style: EelTextStyle.bodyMedium, textAlign: TextAlign.center),
          const SizedBox(height: 32),
          EelTextField(
            label: 'Agent ID',
            hintText: 'Enter your agent ID',
            controller: _agentId,
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
          if (error != null) ...[
            const SizedBox(height: 16),
            EelBanner(message: error, severity: EelSeverity.danger),
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
            label: 'Forgot PIN',
            variant: EelButtonVariant.tertiary,
            fullWidth: true,
            // There is no PIN-recovery endpoint. Rather than a live-looking
            // button that does nothing, it says so.
            onPressed: busy
                ? null
                : () => _report(
                    'PIN resets are issued by returning officer; contact your returning officer to have your PIN reissued.',
                  ),
          ),
          const SizedBox(height: 32),
          Text('eel Election Systems © 2026',
              style: EelTextStyle.bodySmall.copyWith(color: colors.onSurfaceMuted),
              textAlign: TextAlign.center),
        ],
      ),
    );
  }
}
