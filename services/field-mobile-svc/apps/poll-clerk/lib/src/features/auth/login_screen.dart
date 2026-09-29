/// Poll Clerk Login Screen (M28 FR-027) — Beautiful, ergonomic, accessible.
/// Uses eel_design: EelScaffold, EelTextField, EelButton, EelResultCard, EelColors, EelSecureScreen.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class LoginScreen extends ConsumerWidget {
  const LoginScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final colors = EelColors.of(context);
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
          const EelTextField(
            label: 'User ID',
            hintText: 'Enter your clerk ID',
            enabled: true,
          ),
          const SizedBox(height: 16),
          const EelTextField(
            label: 'Password',
            hintText: 'Enter your password',
            obscureText: true,
          ),
          const SizedBox(height: 24),
          EelButton(
            label: 'Sign In',
            icon: Icons.login,
            fullWidth: true,
            onPressed: () {},
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Forgot Password',
            variant: EelButtonVariant.tertiary,
            fullWidth: true,
            onPressed: () {},
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