/// RTS Scan Result — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class ScanResultScreen extends ConsumerWidget {
  const ScanResultScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Scan Result',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: ListView(
        children: [
          const Text('RFID Scan Result', style: EelTextStyle.titleLarge),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.check_circle, color: EelColors.of(context).primary, size: 32),
            title: Text('Scan Verified', style: EelTextStyle.titleMedium),
            subtitle: Text('RFID tag: RT-001 • Station: KE-NBI-001', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Confirm Scan',
            icon: Icons.check,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}
