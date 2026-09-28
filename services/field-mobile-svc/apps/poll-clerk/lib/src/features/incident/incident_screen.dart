/// Poll Clerk Incident Screen (M28 FR-013) — Beautiful, ergonomic, accessible.
/// Uses eel_design: EelScaffold, EelTextField, EelButton, EelResultCard, EelColors, EelSecureScreen.
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
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
          EelTextField(
            label: 'Incident Description',
            maxLines: 4,
            hintText: 'Describe what happened',
            enabled: true,
          ),
          const SizedBox(height: 16),
          EelTextField(
            label: 'Category',
            hintText: 'Intimidation, Bribery, Turned Away, Other',
            enabled: true,
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Submit Incident',
            icon: Icons.report,
            onPressed: () {},
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
