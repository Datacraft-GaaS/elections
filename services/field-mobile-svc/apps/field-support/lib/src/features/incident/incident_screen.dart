/// Field-Support Incident (M29 FR-011) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
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
          const EelTextField(
            label: 'Incident Description',
            maxLines: 4,
            hintText: 'Describe what happened',
          ),
          const SizedBox(height: 16),
          const EelTextField(
            label: 'Category',
            hintText: 'Intimidation, Bribery, Turned Away, Other',
          ),
          const SizedBox(height: 16),
          const EelButton(
            label: 'Submit Incident',
            icon: Icons.report,
            fullWidth: true,
            // Disabled, not wired: this app has no incident controller or client at all.
            //
            // This was an empty callback, which renders a live control that does
            // nothing when tapped. EelButton treats a null callback as disabled,
            // so the unavailable state is now visible rather than silent.
            onPressed: null,
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