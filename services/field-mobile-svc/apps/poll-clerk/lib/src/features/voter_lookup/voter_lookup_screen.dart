/// Voter Lookup (M28 FR-009) — Elegant, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class VoterLookupScreen extends ConsumerWidget {
  const VoterLookupScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Voter Lookup',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      body: ListView(
        children: [
          const EelTextField(
            label: 'Search voter',
            hintText: 'Enter voter ID or name',
            enabled: true,
          ),
          const SizedBox(height: 24),
          EelResultCard(
            leading: Icon(Icons.verified, color: EelColors.of(context).primary, size: 32),
            title: const Text('Voter Verified', style: EelTextStyle.titleMedium),
            subtitle: const Text('Station KE-NBI-001 • Registered', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Confirm Lookup',
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}