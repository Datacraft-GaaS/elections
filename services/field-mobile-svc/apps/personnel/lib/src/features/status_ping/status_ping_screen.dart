/// Personnel Status Ping (M29 FR-014) — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverprovider.dart';
import 'package:eel_design/eel_design.dart';

class StatusPingScreen extends ConsumerWidget {
  const StatusPingScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Status Ping',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Status Ping', style: EelTextStyle.titleLarge),
          const SizedBox(height: 24),
          EelResultCard(
            leading: Icon(Icons.cell_tower, color: EelColors.of(context).primary, size: 32),
            title: Text('Status: Online', style: EelTextStyle.titleMedium),
            subtitle: Text('Location: KE-NBI-001', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Send Status',
            icon: Icons.send,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}