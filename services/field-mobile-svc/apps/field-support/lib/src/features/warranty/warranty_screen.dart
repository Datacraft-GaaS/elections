/// Field-Support Warranty (M29 FR-012) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class WarrantyScreen extends ConsumerWidget {
  const WarrantyScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Warranty',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretched,
        children: [
          const Text('Warranty Status', style: EelTextStyle.titleLarge),
          const SizedBox(height: 24),
          EelResultCard(
            leading: Icon(Icons.shield, color: EelColors.of(context).primary, size: 32),
            title: const Text('Equipment Warranty', style: EelTextStyle.titleMedium),
            subtitle: const Text('All equipment under warranty', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'View Warranty Details',
            icon: Icons.info,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}