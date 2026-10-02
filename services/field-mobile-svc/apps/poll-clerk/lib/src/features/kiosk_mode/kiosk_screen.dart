/// Kiosk Mode (M28 FR-008) — Elegant, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class KioskScreen extends ConsumerWidget {
  const KioskScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Kiosk Mode',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          const Text('Kiosk Mode', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 24),
          EelResultCard(
            leading: Icon(Icons.terminal, color: EelColors.of(context).primary, size: 32),
            title: const Text('Kiosk Active', style: EelTextStyle.titleMedium),
            // Was `Operational - 100% uptime`, a literal. An uptime figure nobody
            // measured is a claim about the availability of the election.
            subtitle: const Text(
                'No health source is wired into this app, so uptime is not measured.',
                style: EelTextStyle.bodyMedium,
            ),
          ),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.analytics, color: EelColors.of(context).primaryMuted, size: 32),
            title: const Text('Kiosk Metrics', style: EelTextStyle.titleMedium),
            subtitle: const Text('Voter turnout trend', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.list_alt, color: EelColors.of(context).primaryMuted, size: 32),
            title: const Text('Report Summary', style: EelTextStyle.titleMedium),
            // Was `Total voters: 12,450 • Turnout: 87% • EOD: 18:00`, a literal.
            // A turnout figure on a report summary is read as a count of voters.
            subtitle: const Text(
                'Unavailable — no tally source is wired into this app.',
                style: EelTextStyle.bodyMedium,
            ),
          ),
          const SizedBox(height: 16),
          const EelButton(
            label: 'Exit Kiosk',
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