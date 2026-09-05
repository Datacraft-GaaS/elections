/// Personnel Incident (M29 FR-018) — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverprovider.dart';
import 'package:eel_design/eel_design.dart';

class IncidentScreen extends ConsumerWidget {
  const IncidentScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Report Incident',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(24),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Report Incident', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 24),
          EelTextField(
            label: 'Incident Description',
            maxLines: 4,
            hintText: 'Describe what happened',
          ),
          const SizedBox(height: 16),
          EelTextField(
            label: 'Category',
            hintText: 'Intimidation, Bribery, Turned Away, Other',
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Submit Incident',
            icon: Icons.report,
            fullWidth: true,
            onPressed: () {},
          ),
          const SizedBox(height: 12),
          EelButton(
            label: 'Cancel',
            variant: EelButtonVariant.outlined,
            fullWidth: true,
            onPressed: () => Navigator.pop(context),
          ),
        ],
      ),
    );
  }
}
