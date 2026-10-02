/// Station Visit (M34 FR-024) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class StationVisitScreen extends ConsumerWidget {
  const StationVisitScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return const EelScaffold(
      title: 'Station Visit',
      variant: EelScaffoldVariant.scrollable,
      padding: EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text('Record Station Visit', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          SizedBox(height: 24),
          EelTextField(
            label: 'Station Code',
            hintText: 'Enter station code',
          ),
          SizedBox(height: 16),
          EelTextField(
            label: 'Arrival Time',
            hintText: 'Select time',
          ),
          SizedBox(height: 16),
          EelTextField(
            label: 'Notes',
            maxLines: 3,
            hintText: 'Add visit notes',
          ),
          SizedBox(height: 16),
          EelButton(
            label: 'Submit Visit',
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