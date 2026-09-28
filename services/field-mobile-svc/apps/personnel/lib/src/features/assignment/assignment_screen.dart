/// Personnel Assignment (M29 FR-015) — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class AssignmentScreen extends ConsumerWidget {
  const AssignmentScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Assignment',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('My Assignment', style: EelTextStyle.titleLarge),
          const SizedBox(height: 24),
          EelResultCard(
            leading: Icon(Icons.assignment, color: EelColors.of(context).primary, size: 32),
            title: Text('Current Assignment', style: EelTextStyle.titleMedium),
            subtitle: Text('Station KE-NBI-001 • Poll Clerk', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelStatusChip(label: 'Status: Active', severity: EelSeverity.success),
          const SizedBox(height: 8),
          EelStatusChip(label: 'Shift: 06:00 - 18:00', severity: EelSeverity.info),
        ],
      ),
    );
  }
}
