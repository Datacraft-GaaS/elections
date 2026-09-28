/// Queue Screen (M28 FR-010) — Elegant, ergonomic, accessible.
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
          EelStatusChip(label: 'Total in queue: 5', severity: EelSeverity.success),
          const SizedBox(height: 8),
          EelStatusChip(label: 'Avg wait: 4 min', severity: EelSeverity.success),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.list, color: EelColors.of(context).primaryMuted, size: 32),
            title: Text('Queue Position', style: EelTextStyle.titleSmall),
            subtitle: Text('Position 3 of 5 • Turnout: 45%', style: EelTextStyle.bodyMedium),
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