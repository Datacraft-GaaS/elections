/// Diaspora Kit Station Screen — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverprovider.dart';
import 'package:eel_design/eel_design.dart';

class StationScreen extends ConsumerWidget {
  const StationScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Diaspora Station',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: ListView(
        children: [
          const Text('Station Overview', style: EelTextStyle.titleLarge),
          const SizedBox(height: 16),
          EelStatusChip(label: 'Connection: Online', severity: 'ok'),
          const SizedBox(height: 8),
          EelStatusChip(label: 'Forms synced: Yes', severity: 'ok'),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.location_on, color: EelColors.of(context).primary, size: 32),
            title: Text('Station KE-DIAS-001', style: EelTextStyle.titleMedium),
            subtitle: Text('Diaspora polling station • 06:00 - 18:00', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Refresh Station Data',
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}