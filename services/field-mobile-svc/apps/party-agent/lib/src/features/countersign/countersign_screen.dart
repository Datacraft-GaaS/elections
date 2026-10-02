/// Party-Agent Countersign (M30 FR-005) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class CountersignScreen extends ConsumerWidget {
  const CountersignScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return const EelScaffold(
      title: 'Countersign',
      variant: EelScaffoldVariant.scrollable,
      padding: EdgeInsets.all(24),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text('Countersign Document', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          SizedBox(height: 24),
          EelTextField(
            label: 'Document ID',
            hintText: 'Enter document ID to countersign',
          ),
          SizedBox(height: 16),
          EelTextField(
            label: 'Countersignature',
            hintText: 'Enter your countersignature',
          ),
          SizedBox(height: 24),
          EelButton(
            label: 'Submit Countersign',
            icon: Icons.check,
            fullWidth: true,
            // Disabled, not wired: this screen has no domain client for the call it describes.
            //
            // This was an empty callback, which renders a live control that does
            // nothing when tapped. EelButton treats a null callback as disabled,
            // so the unavailable state is now visible rather than silent.
            onPressed: null,
          ),
        ],
      ),
    );
  }
}