/// Party-Agent Duress Alert (M30 FR-007) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class DuressAlertScreen extends ConsumerWidget {
  const DuressAlertScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Duress Alert',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(24),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Send Duress Alert', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 24),
          const EelBanner(
            message: 'This will notify security and authorities. Use only in genuine distress.',
            severity: EelSeverity.danger,
          ),
          const SizedBox(height: 24),
          EelResultCard(
            leading: Icon(Icons.warning, color: EelColors.of(context).primary, size: 32),
            title: const Text('Confirm Alert', style: EelTextStyle.titleMedium),
            subtitle: const Text('Press below to send duress signal.', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'SEND DURESS ALERT',
            icon: Icons.emergency,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}