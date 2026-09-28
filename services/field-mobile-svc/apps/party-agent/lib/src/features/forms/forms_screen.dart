/// Party-Agent Forms (M34 FR-021) — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class FormsScreen extends ConsumerWidget {
  const FormsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Forms',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Form Library', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 24),
          EelResultCard(
            leading: Icon(Icons.description, color: EelColors.of(context).primary, size: 32),
            title: Text('Available Forms', style: EelTextStyle.titleMedium),
            subtitle: Text('Select a form to fill or view', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelChips(
            options: const ['Voter\'s Register', 'Candidate Application', 'Dispute Form'],
            onSelected: (chip) {},
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Create New Form',
            icon: Icons.add,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}