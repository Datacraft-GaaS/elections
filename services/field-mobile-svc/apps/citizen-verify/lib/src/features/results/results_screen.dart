/// Results Browser (M13 FR-C5/C6) — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class ResultsScreen extends ConsumerWidget {
  const ResultsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Results',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      body: ListView(
        children: [
          const Text('Election Results — Verified', style: EelTextStyle.titleLarge),
          const SizedBox(height: 8),
          EelStatusChip(label: 'National • County • Constituency • Station', severity: EelSeverity.success),
          const SizedBox(height: 16),
          EelTextField(
            label: 'Station code',
            hintText: 'KE-NBI-001',
          ),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.verified, color: EelColors.of(context).primary, size: 32),
            title: Text('Station KE-NBI-001', style: EelTextStyle.titleMedium),
            subtitle: Text('Turnout: 1,250 / Expected: 2,000 • 87% • Verified', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Load Results',
            icon: Icons.refresh,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}