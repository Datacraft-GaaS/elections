/// Field-Support Diagnostics (M29 FR-010) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class DiagnosticsScreen extends ConsumerWidget {
  const DiagnosticsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Diagnostics',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Kit Diagnostics', style: EelTextStyle.titleLarge),
          const SizedBox(height: 24),
          const EelStatusChip(label: 'Kit Health: OK', severity: EelSeverity.success),
          const SizedBox(height: 8),
          const EelStatusChip(label: 'Battery: 78%', severity: EelSeverity.success),
          const SizedBox(height: 8),
          const EelStatusChip(label: 'Firmware: Current', severity: EelSeverity.success),
          const SizedBox(height: 16),
          EelButton(
            label: 'Run Diagnostics',
            icon: Icons.settings,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}
