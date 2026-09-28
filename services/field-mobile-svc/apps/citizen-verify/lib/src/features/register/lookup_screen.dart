/// Register Lookup (M13 FR-010/011/012) — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class RegisterLookupScreen extends ConsumerWidget {
  const RegisterLookupScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Register Lookup',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      body: ListView(
        children: [
          const Text('Own Record — DPO Attested', style: EelTextStyle.titleLarge),
          const SizedBox(height: 8),
          EelBanner(
            message: 'Requires DPO biometric attestation + written justification per M13 FR-010.',
            severity: EelSeverity.info,
          ),
          const SizedBox(height: 16),
          EelTextField(
            label: 'ID Hash (B)',
            hintText: 'Enter your hashed voter ID — never raw identity',
          ),
          const SizedBox(height: 16),
          EelTextField(
            label: 'Justification / DPO Attestation',
            hintText: 'Required per M13 FR-010',
            maxLines: 3,
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Verify Registration',
            icon: Icons.search,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}
