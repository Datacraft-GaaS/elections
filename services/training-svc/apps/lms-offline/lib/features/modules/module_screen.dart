/// LMS Offline Modules — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverprovider.dart';
import 'package:eel_design/eel_design.dart';

class ModuleScreen extends ConsumerWidget {
  const ModuleScreen({super.key, this.moduleId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Module',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: ListView(
        children: [
          const Text('Module Details', style: EelTextStyle.titleLarge),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.book, color: EelColors.of(context).primary, size: 32),
            title: Text('Module Content', style: EelTextStyle.titleMedium),
            subtitle: Text('Training material and assessment', style: EelTextStyle.bodyMedium),
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
