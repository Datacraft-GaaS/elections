/// Accred-Scanner Scan Screen — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class ScanScreen extends ConsumerWidget {
  const ScanScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Scan Accreditation',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: ListView(
        children: [
          const Text('Scan Accreditation', style: EelTextStyle.titleLarge),
          const SizedBox(height: 16),
          const EelTextField(
            label: 'Scan Code',
            hintText: 'Enter accreditation code',
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Scan',
            icon: Icons.camera_alt,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}