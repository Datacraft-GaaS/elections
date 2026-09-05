/// Party-Agent Check-In (M30 FR-001) — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class CheckInScreen extends ConsumerWidget {
  const CheckInScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Check-In',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(24),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Agent Check-In', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 24),
          EelTextField(
            label: 'Station Code',
            hintText: 'Enter your station code',
          ),
          const SizedBox(height: 16),
          EelTextField(
            label: 'Agent ID',
            hintText: 'Enter your agent ID',
          ),
          const SizedBox(height: 24),
          EelButton(
            label: 'Check In',
            icon: Icons.login,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}
