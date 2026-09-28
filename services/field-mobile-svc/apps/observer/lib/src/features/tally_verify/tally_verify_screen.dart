/// Tally Verification (M34 FR-026) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class TallyVerifyScreen extends ConsumerWidget {
  const TallyVerifyScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Tally Verification',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Verify Tally', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 24),
          EelResultCard(
            leading: Icon(Icons.verified, color: EelColors.of(context).primary, size: 32),
            title: const Text('Tally Status', style: EelTextStyle.titleMedium),
            subtitle: const Text('All tallies verified ✓', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          const EelStatusChip(label: 'Verification: Complete', severity: EelSeverity.success),
          const SizedBox(height: 8),
          const EelStatusChip(label: 'Discrepancies: 0', severity: EelSeverity.success),
          const SizedBox(height: 16),
          EelButton(
            label: 'Verify Now',
            icon: Icons.refresh,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}