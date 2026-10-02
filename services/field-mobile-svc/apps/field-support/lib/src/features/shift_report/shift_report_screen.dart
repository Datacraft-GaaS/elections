/// Field Support Shift Report (M29 FR-014) — Beautiful, ergonomic, accessible.
/// Uses eel_design: EelScaffold, EelResultCard, EelButton, EelChips, EelStatusChip, EelSecureScreen.
library;
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
      // EelScaffold's scrollable variant already provides the viewport, so a
      // ListView here is a scrollable inside a scrollable and throws on
      // performResize. A Column is what the variant expects.
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // EelSecureScreen wrapper per §3.4 / incident closure pattern.
          EelSecureScreen(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Summary overview.
                EelResultCard(
                  leading:
                      Icon(Icons.check_circle, color: colors.primary, size: 32),
                  title:
                      const Text('Shift Complete', style: EelTextStyle.titleMedium),
                  subtitle: const Text(
                      '12 electors processed, all signatures verified',
                      style: EelTextStyle.bodyMedium),
                ),
                const SizedBox(height: 16),
                // Diagnostics card.
                EelResultCard(
                  leading: Icon(Icons.troubleshoot,
                      color: colors.primaryMuted, size: 32),
                  title: const Text('Diagnostics', style: EelTextStyle.titleMedium),
                  // Was `Kit health: OK • Battery: 78% • Firmware current`, a
                  // literal, on a shift report a supervisor signs off against.
                  subtitle: const Text(
                      'Not measured — no health source is wired into this app.',
                      style: EelTextStyle.bodyMedium),
                ),
                const SizedBox(height: 16),
                // Open tickets / incidents.
                EelResultCard(
                  leading: Icon(Icons.task_alt,
                      color: colors.primaryMuted, size: 32),
                  title: const Text('Open Items', style: EelTextStyle.titleMedium),
                  subtitle: const Text('3 tickets pending — 1 standard, 2 urgent',
                      style: EelTextStyle.bodyMedium),
                ),
                const SizedBox(height: 16),
                // Quick actions.
                EelChips(
                  options: const [
                    ChipOption(label: 'Finalize Report'),
                    ChipOption(label: 'Submit to BFF'),
                    ChipOption(label: 'Cancel'),
                  ],
                  onSelected: (_) {},
                ),
                const SizedBox(height: 16),
                const EelButton(
                  label: 'Finalize Shift Report',
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
          ),
        ],
      ),
    );
  }
}
