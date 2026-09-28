/// Diaspora Kit Form 34A Screen — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class Form34aScreen extends ConsumerWidget {
  const Form34aScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Form 34A',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      body: ListView(
        children: [
          const Text('Form 34A — Presiding Officer Report', style: EelTextStyle.titleLarge),
          const SizedBox(height: 8),
          EelStatusChip(label: 'Sealed and signed', severity: EelSeverity.success),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.description, color: EelColors.of(context).primary, size: 32),
            title: Text('Form 34A — Section A', style: EelTextStyle.titleMedium),
            subtitle: Text('Station code, opening time, ballot reconciliation', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 12),
          EelResultCard(
            leading: Icon(Icons.description, color: EelColors.of(context).primary, size: 32),
            title: Text('Form 34A — Section B', style: EelTextStyle.titleMedium),
            subtitle: Text('Vote tallies by candidate', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Verify All Sections',
            icon: Icons.verified,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}
