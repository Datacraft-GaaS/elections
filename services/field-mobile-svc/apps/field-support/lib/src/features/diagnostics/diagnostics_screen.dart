/// Field-Support Diagnostics (M29 FR-010) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class DiagnosticsScreen extends ConsumerWidget {
  const DiagnosticsScreen({super.key, this.kitSerial});

  /// The kit this deep link names. Null when the screen is opened without one.
  final String? kitSerial;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Diagnostics',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Kit Diagnostics', style: EelTextStyle.titleLarge),
          if (kitSerial != null) ...[
            const SizedBox(height: 4),
            Text(kitSerial!, style: EelTextStyle.bodySmall),
          ],
          const SizedBox(height: 24),
          // These were `Kit Health: OK`, `Battery: 78%` and `Firmware: Current`
          // as literals. A kit-health screen that always says OK is worse than one
          // that says it has not checked: the first tells an officer the kit is
          // ready when nothing has been read from it.
          const EelStatusChip(
            label: 'Kit health not read — no health source is wired into this app.',
            severity: EelSeverity.warning,
          ),
          const SizedBox(height: 16),
          const EelButton(
            label: 'Run Diagnostics',
            icon: Icons.settings,
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
