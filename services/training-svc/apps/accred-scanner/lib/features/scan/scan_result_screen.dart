/// Accred-Scanner Scan Result — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

import '../crl/crl_sync_service.dart';
import 'scan_result.dart';

class ScanResultScreen extends HookConsumerWidget {
  final ScanResult result;

  const ScanResultScreen({super.key, required this.result});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isCrlStale = ref.watch(crlStaleProvider);

    return EelScaffold(
      title: 'Scan Result',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _StatusIndicator(status: result.status),
          const SizedBox(height: 24),
          EelResultCard(
            leading: Icon(Icons.tag, color: EelColors.of(context).primary, size: 24),
            title: Text('Serial: ${result.serial}', style: EelTextStyle.bodyMedium),
            subtitle: Text('Checked: ${result.checkedAt.toLocal()}', style: EelTextStyle.bodySmall),
          ),
          if (result.isFromCacheOnly) ...[
            const SizedBox(height: 8),
            const EelBanner(
              message: 'From cache (offline) — using cached CRL data.',
              severity: EelSeverity.warning,
            ),
          ],
          if (isCrlStale) ...[
            const SizedBox(height: 8),
            const EelBanner(
              message: 'CRL is over 30 minutes old. Revocation data may be outdated.',
              severity: EelSeverity.danger,
            ),
          ],
          if (result.zones.isNotEmpty) ...[
            const SizedBox(height: 16),
            const Text('Access Zones', style: EelTextStyle.titleMedium),
            const SizedBox(height: 8),
            ...result.zones.map(
              (zone) => EelResultCard(
                leading: Icon(Icons.location_on, color: EelColors.of(context).primary, size: 24),
                title: Text(zone, style: EelTextStyle.bodyMedium),
              ),
            ),
          ] else
            const EelBanner(message: 'No zone information available.', severity: EelSeverity.info),
          const Spacer(),
          EelButton(
            label: 'Scan Another',
            icon: Icons.qr_code_scanner,
            fullWidth: true,
            onPressed: () => context.go('/scan'),
          ),
        ],
      ),
    );
  }
}

class _StatusIndicator extends StatelessWidget {
  final ScanStatus status;

  const _StatusIndicator({required this.status});

  @override
  Widget build(BuildContext context) {
    final (label, color, icon) = switch (status) {
      ScanStatus.active => ('ACTIVE', Colors.green, Icons.check_circle),
      ScanStatus.revoked => ('REVOKED', Colors.red, Icons.cancel),
      ScanStatus.suspended => ('SUSPENDED', Colors.amber, Icons.pause_circle),
    };

    return Container(
      padding: const EdgeInsets.symmetric(vertical: 32),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.15),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: color, width: 2),
      ),
      child: Column(
        children: [
          Icon(icon, size: 72, color: color),
          const SizedBox(height: 16),
          Text(label,
              style: EelTextStyle.displayMedium.copyWith(
                color: color,
                fontWeight: FontWeight.bold,
              )),
        ],
      ),
    );
  }
}
