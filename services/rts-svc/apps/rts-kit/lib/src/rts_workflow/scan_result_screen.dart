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
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('RFID Scan Result', style: EelTextStyle.titleLarge),
          const SizedBox(height: 16),
          // This rendered a fixed 'Scan Verified' card naming tag RT-001 at
          // station KE-NBI-001. Neither was read from anything, so a completed
          // tally or transfer could be confirmed against a scan that never
          // happened.
          const EelResultCard(
            leading: Icon(Icons.help_outline, size: 32),
            title: Text('No scan recorded', style: EelTextStyle.titleMedium),
            subtitle: Text(
              'Nothing has been scanned. Tag and station are read from a reader, '
              'which is not implemented.',
              style: EelTextStyle.bodyMedium,
            ),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Confirm Scan',
            icon: Icons.check,
            fullWidth: true,
            onPressed: () => ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(
                content: Text('There is no scan to confirm.'),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
