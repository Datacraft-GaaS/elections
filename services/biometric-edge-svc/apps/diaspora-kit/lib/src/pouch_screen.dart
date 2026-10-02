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

/// Diaspora Kit Pouch Screen — Beautiful, ergonomic, accessible.
import 'package:eel_design/eel_design.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import 'providers.dart';

class PouchScreen extends ConsumerWidget {
  const PouchScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Read from the notifier rather than hardcoding: the screen previously
    // rendered two cards reading "Ballot Box (1) • Verified • Sealed" and
    // "Form 34A (2) • Verified • Sealed" under a banner stating the contents
    // "are sealed and verified by BVI back-end". None of that came from
    // anywhere, and PouchNotifier has no loader yet, so the honest state is
    // that no pouch is loaded.
    final pouches = ref.watch(pouchesProvider);

    return EelScaffold(
      title: 'Pouch Contents',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Sealed Pouch Contents', style: EelTextStyle.titleLarge),
          const SizedBox(height: 8),
          if (pouches.isEmpty)
            const EelBanner(
              message:
                  'No pouch is loaded. Sealed contents are read from the BVI '
                  'back-end, which is not wired to this screen yet, so nothing '
                  'here is verified.',
              severity: EelSeverity.warning,
            )
          else ...[
            const EelBanner(
              message:
                  'Loaded from this device. A seal is verified against the BVI '
                  'back-end, which is not wired yet.',
              severity: EelSeverity.warning,
            ),
            const SizedBox(height: 16),
            for (final pouch in pouches) ...[
              EelResultCard(
                leading: Icon(Icons.inventory,
                    color: EelColors.of(context).primary, size: 32),
                title: Text(pouch.id, style: EelTextStyle.titleMedium),
                subtitle: Text(
                  '${pouch.formIds.length} form(s) • '
                  '${pouch.transfers.length} transfer(s) • not seal-verified',
                  style: EelTextStyle.bodyMedium,
                ),
              ),
              const SizedBox(height: 12),
            ],
          ],
          const SizedBox(height: 16),
          EelButton(
            label: 'Verify Pouch',
            icon: Icons.verified,
            fullWidth: true,
            // Nothing to verify against yet. Saying so beats a button that
            // appears to check a seal it never checks.
            onPressed: () => ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(
                content: Text(
                  'Pouch verification needs the BVI back-end, which is not implemented.',
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
