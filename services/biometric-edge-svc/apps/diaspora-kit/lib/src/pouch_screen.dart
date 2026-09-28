/// Diaspora Kit Pouch Screen — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class PouchScreen extends ConsumerWidget {
  const PouchScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Pouch Contents',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: ListView(
        children: [
          const Text('Sealed Pouch Contents', style: EelTextStyle.titleLarge),
          const SizedBox(height: 8),
          EelBanner(
            message: 'Pouch contents are sealed and verified by BVI back-end.',
            severity: EelSeverity.info,
          ),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.inventory, color: EelColors.of(context).primary, size: 32),
            title: Text('Ballot Box (1)', style: EelTextStyle.titleMedium),
            subtitle: Text('Verified • Sealed', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 12),
          EelResultCard(
            leading: Icon(Icons.description, color: EelColors.of(context).primary, size: 32),
            title: Text('Form 34A (2)', style: EelTextStyle.titleMedium),
            subtitle: Text('Verified • Sealed', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Verify Pouch',
            icon: Icons.verified,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}
