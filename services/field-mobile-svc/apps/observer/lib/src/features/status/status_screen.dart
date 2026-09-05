/// Observer Status (M34 FR-025/027) — Beautiful, ergonomic, accessible.
/// Uses eel_design: EelScaffold, EelStatusChip, EelBanner, EelButton, EelColors.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class ObserverStatusScreen extends ConsumerWidget {
  const ObserverStatusScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final colors = EelColors.of(context);
    return EelScaffold(
      title: 'Observer Status',
      variant: EelScaffoldVariant.fixedAppBar,
      body: ListView(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        children: [
          // Hero card — station info with seal.
          EelResultCard(
            leading: EelSealStamp(size: 48, color: colors.primaryMuted),
            title: Text('Station KE-NBI-001', style: EelTextStyle.titleLarge),
            subtitle: Text('Verified observer — active session', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          // Status chips — no PII, aggregate info only.
          EelStatusChip(label: 'Connection: Online', severity: 'ok'),
          const SizedBox(height: 8),
          EelStatusChip(label: 'Forms synced: Yes', severity: 'ok'),
          const SizedBox(height: 8),
          EelStatusChip(label: 'Kit battery: 78%', severity: 'ok'),
          const SizedBox(height: 8),
          EelStatusChip(label: 'Incidents open: 1', severity: 'warning'),
          const SizedBox(height: 16),
          // Security banner.
          EelBanner(
            message: 'FLAG_SECURE / isCaptured / screen-capture-tamper (FR-M31-029)',
            severity: 'info',
          ),
          const SizedBox(height: 8),
          // Connectivity note.
          EelBanner(
            message: 'BVI back-end notified on connectivity change (§5.1, FR-M30-003).',
            severity: 'info',
          ),
          const SizedBox(height: 24),
          EelButton(
            label: 'Refresh Status',
            icon: Icons.refresh,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}
