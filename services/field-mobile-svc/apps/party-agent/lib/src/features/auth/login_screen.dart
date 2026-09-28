/// Party-Agent Login — Beautiful, ergonomic, accessible.
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
            child: EelSealStamp(size: 64, color: colors.primary),
          ),
          const SizedBox(height: 32),
          const Text('Welcome, Party Agent', style: EelTextStyle.displayMedium, textAlign: TextAlign.center),
          const SizedBox(height: 16),
          const Text('Sign in to access check-in, forms, and countersign tools.',
              style: EelTextStyle.bodyMedium, textAlign: TextAlign.center),
          const SizedBox(height: 32),
          const EelTextField(label: 'Agent ID', hintText: 'Enter your agent ID'),
          const SizedBox(height: 16),
          const EelTextField(label: 'Password', hintText: 'Enter your password', obscureText: true),
          const SizedBox(height: 24),
          EelButton(label: 'Sign In', icon: Icons.login, fullWidth: true, onPressed: () {}),
          const SizedBox(height: 16),
          EelButton(
            label: 'Forgot Password',
            variant: EelButtonVariant.tertiary,
            fullWidth: true,
            onPressed: () {},
          ),
          const Spacer(),
          Text('eel Election Systems © 2026',
              style: EelTextStyle.bodySmall.copyWith(color: colors.onSurfaceMuted),
              textAlign: TextAlign.center),
        ],
      ),
    );
  }
}
