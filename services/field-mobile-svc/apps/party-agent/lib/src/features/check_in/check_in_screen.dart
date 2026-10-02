/// Party-Agent Check-In (M30 FR-001) — Beautiful, ergonomic, accessible.
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
          Text('Agent Check-In', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          SizedBox(height: 24),
          EelTextField(
            label: 'Station Code',
            hintText: 'Enter your station code',
          ),
          SizedBox(height: 16),
          EelTextField(
            label: 'Agent ID',
            hintText: 'Enter your agent ID',
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
