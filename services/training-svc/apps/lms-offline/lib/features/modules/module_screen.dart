/// LMS Offline Modules — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class ModuleScreen extends ConsumerWidget {
  const ModuleScreen({super.key, this.moduleId});

  /// The module being shown; null when opened without one.
  final String? moduleId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Module',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      // EelScaffold's scrollable variant already provides the viewport, so a
      // ListView here is a scrollable inside a scrollable and throws on
      // performResize. A Column is what the variant expects.
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Module Details', style: EelTextStyle.titleLarge),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.book, color: EelColors.of(context).primary, size: 32),
            title: const Text('Module Content', style: EelTextStyle.titleMedium),
            subtitle: const Text('Training material and assessment', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Start Assessment',
            icon: Icons.quiz,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}
