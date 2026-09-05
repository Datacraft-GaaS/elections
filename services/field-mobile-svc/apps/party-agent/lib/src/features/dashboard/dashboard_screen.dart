/// Agent Dashboard (M30) — Beautiful, ergonomic, GPS-verified.
/// Uses eel_design: EelScaffold, EelStatusChip, EelResultCard, EelSealStamp, EelButton, EelChips, EelColors.
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
      body: ListView(
        children: [
          // Hero — party agent identity with seal.
          EelResultCard(
            leading: EelSealStamp(size: 56, color: colors.primary),
            title: Text('Party Agent — Station KE-NBI-001', style: EelTextStyle.titleLarge),
            subtitle: Text('GPS-verified check-in • BVI notified', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          // Status chips.
          EelStatusChip(label: 'Check-in: 06:42', severity: 'ok'),
          const SizedBox(height: 8),
          EelStatusChip(label: 'GPS verified: Yes', severity: 'ok'),
          const SizedBox(height: 8),
          EelStatusChip(label: 'BVI notified: Yes', severity: 'ok'),
          const SizedBox(height: 8),
          EelStatusChip(label: 'Forms synced: Complete', severity: 'ok'),
          const SizedBox(height: 16),
          // Quick actions.
          EelBanner(
            message: 'All counters verified. Ready for tally.',
            severity: 'info',
          ),
          const SizedBox(height: 16),
          // Counter chips (example).
          EelChips(
            options: const ['Turnout', 'Incidents', 'Tally', 'Rollup', 'Dispute'],
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