/// Agent Dashboard (M30) — Beautiful, ergonomic, GPS-verified.
/// Uses eel_design: EelScaffold, EelStatusChip, EelResultCard, EelSealStamp, EelButton, EelChips, EelColors.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class AgentDashboardScreen extends ConsumerWidget {
  const AgentDashboardScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final colors = EelColors.of(context);
    return EelScaffold(
      title: 'Agent Dashboard',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      // EelScaffold's scrollable variant already provides the viewport, so a
      // ListView here is a scrollable inside a scrollable and throws on
      // performResize. A Column is what the variant expects.
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Hero — party agent identity with seal.
          EelResultCard(
            leading: EelSealStamp(size: 56, color: colors.primary),
            title: const Text('Party Agent — Station KE-NBI-001',
                style: EelTextStyle.titleLarge),
            subtitle: const Text('GPS-verified check-in • BVI notified',
                style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          // Status chips.
          const EelStatusChip(
              label: 'Check-in: 06:42', severity: EelSeverity.success),
          const SizedBox(height: 8),
          const EelStatusChip(
              label: 'GPS verified: Yes', severity: EelSeverity.success),
          const SizedBox(height: 8),
          const EelStatusChip(
              label: 'BVI notified: Yes', severity: EelSeverity.success),
          const SizedBox(height: 8),
          const EelStatusChip(
              label: 'Forms synced: Complete', severity: EelSeverity.success),
          const SizedBox(height: 16),
          // Quick actions.
          const EelBanner(
            message: 'All counters verified. Ready for tally.',
            severity: EelSeverity.info,
          ),
          const SizedBox(height: 16),
          // Counter chips (example).
          EelChips(
            options: const [
              ChipOption(label: 'Turnout'),
              ChipOption(label: 'Incidents'),
              ChipOption(label: 'Tally'),
              ChipOption(label: 'Rollup'),
              ChipOption(label: 'Dispute'),
            ],
            onSelected: (chip) {},
          ),
          const SizedBox(height: 16),
          // Primary action.
          EelButton(
            label: 'Start Tally Verification',
            icon: Icons.verified,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}
