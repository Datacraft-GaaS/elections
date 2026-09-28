/// Accred-Scanner Scan Result — Beautiful, ergonomic, accessible.
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
          const Text('Scan Result', style: EelTextStyle.titleLarge),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.verified, color: EelColors.of(context).primary, size: 32),
            title: Text('Accreditation Verified', style: EelTextStyle.titleMedium),
            subtitle: Text('Status: Active • Issued: 2026-08-15', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Confirm',
            icon: Icons.check,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}