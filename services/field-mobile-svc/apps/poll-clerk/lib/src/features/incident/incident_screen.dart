/// Poll Clerk Incident Screen (M28 FR-013) — Beautiful, ergonomic, accessible.
/// Uses eel_design: EelScaffold, EelTextField, EelButton, EelResultCard, EelColors, EelSecureScreen.
library;
import 'package:flutter/material.dart';
import 'package:eel_design/eel_design.dart';

class IncidentScreen extends StatelessWidget {
  const IncidentScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return EelScaffold(
      title: 'Report Incident',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(24),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Report an Incident', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 24),
          const EelTextField(
            label: 'Incident Description',
            maxLines: 4,
            hintText: 'Describe what happened',
            enabled: true,
          ),
          const SizedBox(height: 16),
          const EelTextField(
            label: 'Category',
            hintText: 'Intimidation, Bribery, Turned Away, Other',
            enabled: true,
          ),
          const SizedBox(height: 16),
          const EelButton(
            label: 'Submit Incident',
            icon: Icons.report,
            // Disabled, not wired: this screen has no domain client for the call it describes.
            //
            // This was an empty callback, which renders a live control that does
            // nothing when tapped. EelButton treats a null callback as disabled,
            // so the unavailable state is now visible rather than silent.
            onPressed: null,
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Cancel',
            variant: EelButtonVariant.outlined,
            fullWidth: false,
            onPressed: () => Navigator.pop(context),
          ),
        ],
      ),
    );
  }
}
