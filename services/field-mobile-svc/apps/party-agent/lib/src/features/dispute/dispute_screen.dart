/// Party-Agent Dispute (M30 FR-004) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class DisputeScreen extends ConsumerWidget {
  const DisputeScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Dispute',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(24),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Submit Dispute', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 24),
          const EelTextField(
            label: 'Dispute Description',
            maxLines: 4,
            hintText: 'Describe the dispute',
          ),
          const SizedBox(height: 16),
          const EelTextField(
            label: 'Dispute Type',
            hintText: 'Vote count, Ballot, etc.',
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Submit Dispute',
            icon: Icons.warning,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}