/// EOD Report (M28 FR-007) — Elegant, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class EODReportScreen extends ConsumerWidget {
  const EODReportScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'EOD Report',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: ListView(
        children: [
          const Text('End of Day Report', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.check_circle, color: EelColors.of(context).primary, size: 32),
            title: const Text('Daily Summary', style: EelTextStyle.titleMedium),
            subtitle: const Text('Total voters: 12,450 • Turnout: 87% • EOD: 18:00', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.analytics, color: EelColors.of(context).primaryMuted, size: 32),
            title: const Text('Kiosk Status', style: EelTextStyle.titleMedium),
            subtitle: const Text('Operational - 100% uptime', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.analytics, color: EelColors.of(context).primaryMuted, size: 32),
            title: const Text('EOD Metrics', style: EelTextStyle.titleMedium),
            subtitle: const Text('Voter turnout trend', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.list_alt, color: EelColors.of(context).primaryMuted, size: 32),
            title: const Text('Report Summary', style: EelTextStyle.titleMedium),
            subtitle: const Text('Total voters: 12,450 • Turnout: 87% • EOD: 18:00', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Download PDF',
            icon: Icons.download,
            fullWidth: true,
            onPressed: () {},
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'View Full Report',
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}