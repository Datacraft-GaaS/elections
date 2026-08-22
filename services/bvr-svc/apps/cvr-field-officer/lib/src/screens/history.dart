/// HistoryScreen: paginated list of captures with status badges.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/capture.dart';

class HistoryScreen extends ConsumerWidget {
  const HistoryScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // TODO: load from API via GET /v1/cvr/field-notifications?officer_id=&page=
    const List<CaptureRecord> captures = [];

    return Scaffold(
      appBar: AppBar(title: const Text('Capture History')),
      body: captures.isEmpty
          ? const Center(child: Text('No captures yet.'))
          : ListView.builder(
              itemCount: captures.length,
              itemBuilder: (context, index) {
                final capture = captures[index];
                return Semantics(
                  label:
                      '${_modalityLabel(capture.modality)}, ${_verdictLabel(capture.verdict)}',
                  child: ListTile(
                    leading: Icon(_modalityIcon(capture.modality)),
                    title: Text(_modalityLabel(capture.modality)),
                    subtitle: Text(capture.capturedAt.toLocal().toString()),
                    trailing: _VerdictBadge(verdict: capture.verdict),
                  ),
                );
              },
            ),
    );
  }

  String _modalityLabel(CaptureModality m) => switch (m) {
        CaptureModality.deathCertificate => 'Death Certificate',
        CaptureModality.idCorrection => 'ID Correction',
      };

  IconData _modalityIcon(CaptureModality m) => switch (m) {
        CaptureModality.deathCertificate => Icons.article_outlined,
        CaptureModality.idCorrection => Icons.badge_outlined,
      };

  String _verdictLabel(ReviewVerdict v) => switch (v) {
        ReviewVerdict.pending => 'Pending',
        ReviewVerdict.accepted => 'Accepted',
        ReviewVerdict.rejected => 'Rejected',
        ReviewVerdict.manualReview => 'Manual Review',
      };
}

class _VerdictBadge extends StatelessWidget {
  final ReviewVerdict verdict;

  const _VerdictBadge({required this.verdict});

  @override
  Widget build(BuildContext context) {
    final (label, color) = switch (verdict) {
      ReviewVerdict.pending => ('Pending', Colors.orange),
      ReviewVerdict.accepted => ('Accepted', Colors.green),
      ReviewVerdict.rejected => ('Rejected', Colors.red),
      ReviewVerdict.manualReview => ('Manual', Colors.blue),
    };

    return Chip(
      label: Text(
        label,
        style: const TextStyle(fontSize: 11, color: Colors.white),
      ),
      backgroundColor: color,
      padding: EdgeInsets.zero,
    );
  }
}
