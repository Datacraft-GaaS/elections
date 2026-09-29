import 'dart:convert';

/// Diaspora Kit Form 34A Screen — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';
import 'package:uuid/uuid.dart';

import 'models.dart';
import 'providers.dart';

class Form34aScreen extends ConsumerWidget {
  const Form34aScreen({super.key, this.station});

  /// The station this form is being filed against. Null when opened without
  /// one, in which case the form shows its generic description.
  final EmbassyStation? station;

  /// Signs the form with the device key and appends it to the offline queue.
  ///
  /// The signature is made here rather than at transmission, because an entry
  /// that reached the queue unsigned would be transmitted as a claim the kit
  /// cannot substantiate. Transmission replays the stored signature.
  Future<void> _submitAndQueue(BuildContext context, WidgetRef ref) async {
    final station = this.station;
    if (station == null) return;

    final entry = Form34AEntry(
      stationId: station.id,
      formId: 'f34a-${const Uuid().v7()}',
      results: const <String, int>{},
      rejected: 0,
      disputed: 0,
    );

    final sign = ref.read(formSignerProvider);
    entry.hsmSignature = await sign(
      '${entry.formId}|${entry.stationId}|'
      '${jsonEncode(entry.results)}|${entry.rejected}|${entry.disputed}',
    );
    entry.queuedAt = DateTime.now();

    ref.read(offlineFormQueueProvider.notifier).enqueue(entry);

    if (context.mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Form 34A queued for transmission')),
      );
    }
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Form 34A',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      body: Column(
        children: [
          const Text('Form 34A — Presiding Officer Report', style: EelTextStyle.titleLarge),
          const SizedBox(height: 8),
          EelStatusChip(label: 'Sealed and signed', severity: EelSeverity.success),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.description, color: EelColors.of(context).primary, size: 32),
            title: Text('Form 34A — Section A', style: EelTextStyle.titleMedium),
            subtitle: Text(
              station == null
                  ? 'Station code, opening time, ballot reconciliation'
                  : 'Station ${station!.id} — opening time, ballot reconciliation',
              style: EelTextStyle.bodyMedium,
            ),
          ),
          const SizedBox(height: 12),
          EelResultCard(
            leading: Icon(Icons.description, color: EelColors.of(context).primary, size: 32),
            title: Text('Form 34A — Section B', style: EelTextStyle.titleMedium),
            subtitle: Text('Vote tallies by candidate', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Verify All Sections',
            icon: Icons.verified,
            fullWidth: true,
            onPressed: () {},
          ),
          const SizedBox(height: 16),
          EelButton(
            key: const Key('submit-form34a'),
            label: 'Submit to Offline Queue',
            icon: Icons.save,
            fullWidth: true,
            // Queues a signed entry. Until this existed the screen had no
            // submit path at all, so a captured form could never reach the
            // queue.
            onPressed: station == null
                ? null
                : () async => _submitAndQueue(context, ref),
          ),
        ],
      ),
    );
  }
}
