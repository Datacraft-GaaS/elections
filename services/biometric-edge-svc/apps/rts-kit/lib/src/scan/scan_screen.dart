/// RTS Scan Screen — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class ScanScreen extends ConsumerWidget {
  const ScanScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Scan RFID',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: ListView(
        children: [
          const Text('RFID Scan', style: EelTextStyle.titleLarge),
          const SizedBox(height: 24),
          EelStatusChip(label: 'RFID Reader: Ready', severity: 'ok'),
          const SizedBox(height: 16),
          EelButton(
            label: 'Start Scan',
            icon: Icons.radar,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}