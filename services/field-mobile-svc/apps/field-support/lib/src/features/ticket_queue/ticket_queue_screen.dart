/// Field-Support Ticket Queue (M29 FR-013) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class TicketQueueScreen extends ConsumerWidget {
  const TicketQueueScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Ticket Queue',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Open Tickets', style: EelTextStyle.titleLarge),
          const SizedBox(height: 24),
          const EelStatusChip(label: 'Open: 3', severity: EelSeverity.warning),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.confirmation_num, color: EelColors.of(context).primary, size: 32),
            title: const Text('Ticket #1234', style: EelTextStyle.titleMedium),
            subtitle: const Text('Field support request • Pending', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          const EelButton(
            label: 'Create New Ticket',
            icon: Icons.add,
            fullWidth: true,
            // Disabled, not wired: this screen has no domain client for the call it describes.
            //
            // This was an empty callback, which renders a live control that does
            // nothing when tapped. EelButton treats a null callback as disabled,
            // so the unavailable state is now visible rather than silent.
            onPressed: null,
          ),
        ],
      ),
    );
  }
}