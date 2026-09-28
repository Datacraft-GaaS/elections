/// Party-Agent Rollup (M30 FR-003) — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class RollupScreen extends ConsumerWidget {
  const RollupScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Rollup',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Rollup Dashboard', style: EelTextStyle.titleLarge),
          const SizedBox(height: 24),
          EelStatusChip(label: 'Status: Ready for Tally', severity: EelSeverity.success),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.summarize, color: EelColors.of(context).primary, size: 32),
            title: Text('Rollup Complete', style: EelTextStyle.titleMedium),
            subtitle: Text('All forms consolidated', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Submit Rollup',
            icon: Icons.send,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}