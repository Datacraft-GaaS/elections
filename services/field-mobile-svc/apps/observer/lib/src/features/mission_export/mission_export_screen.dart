/// Mission Export (M34 FR-023) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class MissionExportScreen extends ConsumerWidget {
  const MissionExportScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Mission Export',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      // EelScaffold's scrollable variant already provides the viewport, so a
      // ListView here is a scrollable inside a scrollable and throws on
      // performResize. A Column is what the variant expects.
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Export Mission Data', style: EelTextStyle.titleLarge),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.upload_file,
                color: EelColors.of(context).primary, size: 32),
            title: const Text('Export Options', style: EelTextStyle.titleMedium),
            subtitle: const Text('Choose the data format and scope',
                style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelChips(
            options: const [
              ChipOption(label: 'CSV'),
              ChipOption(label: 'PDF'),
              ChipOption(label: 'JSON'),
            ],
            onSelected: (chip) {},
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Generate Export',
            icon: Icons.download,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}
