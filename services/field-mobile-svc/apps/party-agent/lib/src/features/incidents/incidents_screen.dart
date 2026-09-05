/// Party-Agent Incidents (M30 FR-008) — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverprovider.dart';
import 'package:eel_design/eel_design.dart';

class IncidentsScreen extends ConsumerWidget {
  const IncidentsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Incidents',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Incidents', style: EelTextStyle.titleLarge),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.warning, color: EelColors.of(context).primary, size: 32),
            title: Text('Open Incidents', style: EelTextStyle.titleMedium),
            subtitle: Text('3 incidents reported', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Report New Incident',
            icon: Icons.add,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}
