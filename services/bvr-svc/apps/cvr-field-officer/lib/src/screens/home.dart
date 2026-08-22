/// HomeScreen: officer name + shift status + queue depth + last-sync +
/// quick-action buttons for "Capture death cert" and "Capture ID correction".
///
/// WCAG AA: touch targets >= 48 dp, high-contrast colour scheme, semantics.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/capture.dart';
import 'capture.dart';
import 'history.dart';
import 'sync.dart';

class HomeScreen extends ConsumerWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // TODO: load officer profile from kit-sdk attestation and local store.
    const officerName = 'Field Officer';
    const queueDepth = 0;
    final lastSync = DateTime.now().subtract(const Duration(minutes: 5));

    return Scaffold(
      appBar: AppBar(
        title: const Text('CVR Field Officer'),
        actions: [
          Semantics(
            label: 'Sync queue',
            button: true,
            child: IconButton(
              icon: const Icon(Icons.sync),
              tooltip: 'Sync',
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute<void>(builder: (_) => const SyncScreen()),
              ),
            ),
          ),
        ],
      ),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Officer info card
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Semantics(
                        header: true,
                        child: Text(
                          officerName,
                          style: Theme.of(context).textTheme.titleLarge,
                        ),
                      ),
                      const SizedBox(height: 8),
                      Row(
                        children: [
                          const Icon(Icons.pending_actions, size: 16),
                          const SizedBox(width: 4),
                          Text('${queueDepth.toString()} pending'),
                          const SizedBox(width: 16),
                          const Icon(Icons.update, size: 16),
                          const SizedBox(width: 4),
                          Text('Synced ${_formatRelative(lastSync)}'),
                        ],
                      ),
                    ],
                  ),
                ),
              ),

              const SizedBox(height: 24),

              Semantics(
                header: true,
                child: Text(
                  'Quick Actions',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ),
              const SizedBox(height: 12),

              // WCAG: 48dp minimum height enforced on FilledButton
              _ActionButton(
                icon: Icons.article_outlined,
                label: 'Capture Death Certificate',
                onPressed: () => _navigateCapture(
                  context,
                  CaptureModality.deathCertificate,
                ),
              ),
              const SizedBox(height: 12),
              _ActionButton(
                icon: Icons.badge_outlined,
                label: 'Capture ID Correction',
                onPressed: () => _navigateCapture(
                  context,
                  CaptureModality.idCorrection,
                ),
              ),

              const Spacer(),

              // History link
              Center(
                child: TextButton.icon(
                  icon: const Icon(Icons.history),
                  label: const Text('View History'),
                  onPressed: () => Navigator.of(context).push(
                    MaterialPageRoute<void>(
                        builder: (_) => const HistoryScreen()),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  void _navigateCapture(BuildContext context, CaptureModality modality) {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => CaptureScreen(modality: modality),
      ),
    );
  }

  String _formatRelative(DateTime dt) {
    final diff = DateTime.now().difference(dt);
    if (diff.inSeconds < 60) return 'just now';
    if (diff.inMinutes < 60) return '${diff.inMinutes}m ago';
    if (diff.inHours < 24) return '${diff.inHours}h ago';
    return '${diff.inDays}d ago';
  }
}

class _ActionButton extends StatelessWidget {
  final IconData icon;
  final String label;
  final VoidCallback onPressed;

  const _ActionButton({
    required this.icon,
    required this.label,
    required this.onPressed,
  });

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      label: label,
      child: SizedBox(
        width: double.infinity,
        height: 56, // >= 48 dp WCAG AA
        child: FilledButton.icon(
          icon: Icon(icon),
          label: Text(label),
          onPressed: onPressed,
        ),
      ),
    );
  }
}
