/// Onboarding (M29 FR-C1) — elegant, accessible, beautiful first impression.
/// Uses eel_design system: EelScaffold, EelButton, EelResultCard, EelSealStamp.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class OnboardingScreen extends ConsumerWidget {
  const OnboardingScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final colors = EelColors.of(context);
    return EelScaffold(
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 32),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const SizedBox(height: 48),
          // Hero seal — soft attestation stamp, immediately establishes trust.
          Center(
            child: EelSealStamp(
              label: 'eel',
              size: 96,
              color: colors.primary,
            ),
          ),
          const SizedBox(height: 24),
          Text(
            'Verify, find, see.',
            style: EelTextStyle.displayMedium.copyWith(
              color: colors.onSurface,
            ),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 12),
          Text(
            'Your vote, your proof, your device.',
            style: EelTextStyle.titleMedium.copyWith(
              color: colors.onSurfaceMuted,
            ),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 48),
          // Three trust cards.
          _TrustCard(
            icon: Icons.shield_outlined,
            title: 'No tracking',
            body: 'We use no third-party analytics, ads, or trackers. Open-source code.',
          ),
          const SizedBox(height: 12),
          _TrustCard(
            icon: Icons.smartphone_outlined,
            title: 'No PII on server',
            body: 'Phone numbers and ID numbers stay on your device. Server only verifies hashes.',
          ),
          const SizedBox(height: 12),
          _TrustCard(
            icon: Icons.verified_outlined,
            title: 'Cryptographic proof',
            body: 'Results are signed and verified on your device — no server trust required.',
          ),
          // A Spacer is an Expanded, which needs a bounded height. This body
          // sits in EelScaffold's scrollable variant, so the height is
          // unbounded and the Spacer threw during layout.
          const SizedBox(height: 32),
          // Bottom: legal line + actions.
          Text(
            'By continuing, you accept the no-tracking attestation (DPA 2019 §28).',
            style: EelTextStyle.bodySmall.copyWith(
              color: colors.onSurfaceMuted,
            ),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Continue with phone',
            icon: Icons.arrow_forward,
            fullWidth: true,
            onPressed: () => Navigator.of(context).pushReplacementNamed('/otp'),
          ),
          const SizedBox(height: 8),
          EelButton(
            label: 'Skip — just browse results',
            variant: EelButtonVariant.tertiary,
            fullWidth: true,
            onPressed: () => Navigator.of(context).pushReplacementNamed('/results'),
          ),
        ],
      ),
    );
  }
}

class _TrustCard extends StatelessWidget {
  const _TrustCard({
    required this.icon,
    required this.title,
    required this.body,
  });

  final IconData icon;
  final String title;
  final String body;

  @override
  Widget build(BuildContext context) {
    final colors = EelColors.of(context);
    return EelResultCard(
      leading: Container(
        width: 48,
        height: 48,
        decoration: BoxDecoration(
          color: colors.primarySurface,
          borderRadius: BorderRadius.circular(12),
        ),
        child: Icon(icon, color: colors.primary, size: 24),
      ),
      title: Text(title, style: EelTextStyle.titleMedium),
      subtitle: Text(body, style: EelTextStyle.bodyMedium.copyWith(color: colors.onSurfaceMuted)),
    );
  }
}
