/// Queue Screen (M28 FR-010) — Elegant, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class QueueScreen extends ConsumerWidget {
  const QueueScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Queue',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text('Queue Management', style: EelTextStyle.titleLarge),
          const SizedBox(height: 16),
          const EelStatusChip(label: 'Total in queue: 5', severity: EelSeverity.success),
          const SizedBox(height: 8),
          const EelStatusChip(label: 'Avg wait: 4 min', severity: EelSeverity.success),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.list, color: EelColors.of(context).primaryMuted, size: 32),
            title: const Text('Queue Position', style: EelTextStyle.titleSmall),
            subtitle: const Text('Position 3 of 5 • Turnout: 45%', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Refresh Queue',
            icon: Icons.refresh,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}