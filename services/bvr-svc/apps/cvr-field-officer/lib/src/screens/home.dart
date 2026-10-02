// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// HomeScreen: officer name + shift status + queue depth + last-sync +
/// quick-action buttons for "Capture death cert" and "Capture ID correction".
///
/// WCAG AA: touch targets >= 48 dp, high-contrast colour scheme, semantics.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart' show AttestationVerdict;

import '../models/capture.dart';
import '../services/app_identity.dart';
import '../services/offline_queue.dart';
import 'capture.dart';
import 'history.dart';
import 'sync.dart';

class HomeScreen extends ConsumerWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final queue = OfflineQueueManager();
    final queueDepth = queue.pendingCount;
    final lastSync = queue.lastSyncAt;

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
                        child: _officerName(context),
                      ),
                      const SizedBox(height: 8),
                      Row(
                        children: [
                          const Icon(Icons.pending_actions, size: 16),
                          const SizedBox(width: 4),
                          Text('$queueDepth pending'),
                          const SizedBox(width: 16),
                          const Icon(Icons.update, size: 16),
                          const SizedBox(width: 4),
                          Text(lastSync == null
                              ? 'Never synced'
                              : 'Synced ${_formatRelative(lastSync)}'),
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
  /// Officer display name from the kit-sdk attestation verdict. Falls back
  /// to a generic label until the device attests (or when attestation is
  /// unavailable on this device).
  Widget _officerName(BuildContext context) {
    return FutureBuilder<AttestationVerdict>(
      future: AppIdentity.instance.ensureAttested(),
      builder: (context, snap) {
        final verdict = snap.data;
        final name = verdict == null ? 'Field Officer' : verdict.deviceId;
        return Text(name, style: Theme.of(context).textTheme.titleLarge);
      },
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
