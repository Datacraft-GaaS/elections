/// Kiosk Mode (M28 FR-008) — Elegant, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverprovider.dart';
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
            leading: Icon(Icons.kiosk, color: EelColors.of(context).primary, size: 32),
            title: Text('Kiosk Active', style: EelTextStyle.titleMedium),
            subtitle: Text('Operational - 100% uptime', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.analytics, color: EelColors.of(context).primaryMuted, size: 32),
            title: Text('Kiosk Metrics', style: EelTextStyle.titleMedium),
            subtitle: Text('Voter turnout trend', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.list_alt, color: EelColors.of(context).primaryMuted, size: 32),
            title: Text('Report Summary', style: EelTextStyle.titleMedium),
            subtitle: Text('Total voters: 12,450 • Turnout: 87% • EOD: 18:00', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Exit Kiosk',
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}