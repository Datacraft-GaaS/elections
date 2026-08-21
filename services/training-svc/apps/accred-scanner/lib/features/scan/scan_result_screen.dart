import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import '../crl/crl_sync_service.dart';
import 'scan_result.dart';

class ScanResultScreen extends HookConsumerWidget {
  final ScanResult result;

  const ScanResultScreen({super.key, required this.result});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isCrlStale = ref.watch(crlStaleProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Scan Result'),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go('/scan'),
        ),
      ),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            _StatusIndicator(status: result.status),
            const SizedBox(height: 24),
            Text(
              'Serial: ${result.serial}',
              style: Theme.of(context).textTheme.bodyMedium,
            ),
            Text(
              'Checked: ${result.checkedAt.toLocal()}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            if (result.isFromCacheOnly) ...[
              const SizedBox(height: 8),
              const Chip(
                label: Text('From cache (offline)'),
                backgroundColor: Colors.amber,
              ),
            ],
            if (isCrlStale) ...[
              const SizedBox(height: 8),
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Colors.red.shade100,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: const Row(
                  children: [
                    Icon(Icons.warning, color: Colors.red),
                    SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        'CRL is over 30 minutes old. Revocation data may be outdated.',
                        style: TextStyle(color: Colors.red),
                      ),
                    ),
                  ],
                ),
              ),
            ],
            const SizedBox(height: 24),
            if (result.zones.isNotEmpty) ...[
              Text(
                'Access Zones',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 8),
              ...result.zones.map(
                (zone) => ListTile(
                  leading: const Icon(Icons.location_on),
                  title: Text(zone),
                  dense: true,
                ),
              ),
            ] else
              const Text('No zone information available.'),
            const Spacer(),
            OutlinedButton.icon(
              icon: const Icon(Icons.qr_code_scanner),
              label: const Text('Scan Another'),
              onPressed: () => context.go('/scan'),
            ),
          ],
        ),
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
          Text(
            label,
            style: Theme.of(context).textTheme.displaySmall?.copyWith(
                  color: color,
                  fontWeight: FontWeight.bold,
                ),
          ),
        ],
      ),
    );
  }
}
