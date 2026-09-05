/// Incident Report (M29 FR-C7) — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverprovider.dart';
import 'package:eel_design/eel_design.dart';

class IncidentReportScreen extends ConsumerWidget {
  const IncidentReportScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Report Incident',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      body: ListView(
        children: [
          const Text('Anonymous Incident Report', style: EelTextStyle.titleLarge),
          const SizedBox(height: 8),
          const EelBanner(
            message: 'Your report is anonymous. No identity is linked to this submission.',
            severity: 'info',
          ),
          const SizedBox(height: 16),
          EelTextField(
            label: 'Describe the incident',
            maxLines: 4,
            hintText: 'What happened? Where?',
          ),
          const SizedBox(height: 16),
          EelChips(
            options: const ['Intimidation', 'Bribery', 'Turned Away', 'Other'],
            onSelected: (chip) {},
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Submit Anonymously',
            icon: Icons.send,
            fullWidth: true,
            onPressed: () => ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(content: Text('Report queued (anonymous, GPS optional)')),
            ),
          ),
        ],
      ),
    );
  }
}