/// LMS Offline Assessment — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class AssessmentScreen extends ConsumerWidget {
  const AssessmentScreen({super.key, this.moduleId});

  final String? moduleId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Assessment',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      // EelScaffold's scrollable variant already provides the viewport, so a
      // ListView here is a scrollable inside a scrollable and throws on
      // performResize. A Column is what the variant expects.
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Module Assessment', style: EelTextStyle.titleLarge),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.quiz,
                color: EelColors.of(context).primary, size: 32),
            title: const Text('Question 1 of 10', style: EelTextStyle.titleMedium),
            subtitle:
                const Text('Select the best answer', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelChips(
            options: const [
              ChipOption(label: 'A'),
              ChipOption(label: 'B'),
              ChipOption(label: 'C'),
              ChipOption(label: 'D'),
            ],
            onSelected: (chip) {},
          ),
          const SizedBox(height: 24),
          EelButton(
            label: 'Next Question',
            icon: Icons.arrow_forward,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}
