/// Party-Agent Countersign (M30 FR-005) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class CountersignScreen extends ConsumerWidget {
  const CountersignScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Countersign',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(24),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Countersign Document', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 24),
          const EelTextField(
            label: 'Document ID',
            hintText: 'Enter document ID to countersign',
          ),
          const SizedBox(height: 16),
          const EelTextField(
            label: 'Countersignature',
            hintText: 'Enter your countersignature',
          ),
          const SizedBox(height: 24),
          EelButton(
            label: 'Submit Countersign',
            icon: Icons.check,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}