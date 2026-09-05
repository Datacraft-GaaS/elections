/// Field Support Shift Report (M29 FR-014) — Beautiful, ergonomic, accessible.
/// Uses eel_design: EelScaffold, EelResultCard, EelButton, EelChips, EelStatusChip, EelSecureScreen.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class ShiftReportScreen extends ConsumerWidget {
  const ShiftReportScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final colors = EelColors.of(context);
    return EelScaffold(
      title: 'Shift Report',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      body: ListView(
        children: [
          // EelSecureScreen wrapper per §3.4 / incident closure pattern.
          EelSecureScreen(
            title: 'Shift Report — Close Shift',
            body: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Summary overview.
                EelResultCard(
                  leading: Icon(Icons.check_circle, color: colors.primary, size: 32),
                  title: Text('Shift Complete', style: EelTextStyle.titleMedium),
                  subtitle: Text('12 electors processed, all signatures verified', style: EelTextStyle.bodyMedium),
                ),
                const SizedBox(height: 16),
                // Diagnostics card.
                EelResultCard(
                  leading: Icon(Icons.diagnostics, color: colors.primaryMuted, size: 32),
                  title: Text('Diagnostics', style: EelTextStyle.titleMedium),
                  subtitle: Text('Kit health: OK • Battery: 78% • Firmware current', style: EelTextStyle.bodyMedium),
                ),
                const SizedBox(height: 16),
                // Open tickets / incidents.
                EelResultCard(
                  leading: Icon(Icons.task_alt, color: colors.primaryMuted, size: 32),
                  title: Text('Open Items', style: EelTextStyle.titleMedium),
                  subtitle: Text('3 tickets pending — 1 standard, 2 urgent', style: EelTextStyle.bodyMedium),
                ),
                const SizedBox(height: 16),
                // Quick actions.
                EelChips(
                  options: const ['Finalize Report', 'Submit to BFF', 'Cancel'],
                  onSelected: (_) {},
                ),
                const SizedBox(height: 16),
                EelButton(
                  label: 'Finalize Shift Report',
                  fullWidth: true,
                  onPressed: () {},
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}