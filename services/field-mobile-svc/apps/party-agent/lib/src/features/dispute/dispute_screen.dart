/// Party-Agent Dispute (M30 FR-004) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class DisputeScreen extends ConsumerWidget {
  const DisputeScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return const EelScaffold(
      title: 'Dispute',
      variant: EelScaffoldVariant.scrollable,
      padding: EdgeInsets.all(24),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text('Submit Dispute', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          SizedBox(height: 24),
          EelTextField(
            label: 'Dispute Description',
            maxLines: 4,
            hintText: 'Describe the dispute',
          ),
          SizedBox(height: 16),
          EelTextField(
            label: 'Dispute Type',
            hintText: 'Vote count, Ballot, etc.',
          ),
          SizedBox(height: 16),
          EelButton(
            label: 'Submit Dispute',
            icon: Icons.warning,
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