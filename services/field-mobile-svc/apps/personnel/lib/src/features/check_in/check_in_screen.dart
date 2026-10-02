/// Personnel Check-In (M29 FR-016) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class CheckInScreen extends ConsumerWidget {
  const CheckInScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return const EelScaffold(
      title: 'Check-In',
      variant: EelScaffoldVariant.scrollable,
      padding: EdgeInsets.all(24),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text('Personnel Check-In', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          SizedBox(height: 24),
          EelTextField(
            label: 'Personnel ID',
            hintText: 'Enter your personnel ID',
          ),
          SizedBox(height: 16),
          EelTextField(
            label: 'Station Code',
            hintText: 'Enter station code',
          ),
          SizedBox(height: 24),
          EelButton(
            label: 'Check In',
            icon: Icons.login,
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
