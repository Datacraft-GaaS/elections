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

/// Queue Screen (M28 FR-010) — Elegant, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

/// Stated on the screen rather than rendered as a figure.
const String unavailable = 'Queue data unavailable';
const String queueUnavailable =
    'Unavailable — no queue source is wired into this app, so nothing here is measured.';

class QueueScreen extends ConsumerWidget {
  const QueueScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Queue',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text('Queue Management', style: EelTextStyle.titleLarge),
          const SizedBox(height: 16),
          // These were `Total in queue: 5`, `Avg wait: 4 min` and
          // `Position 3 of 5 • Turnout: 45%` as literals. An election official
          // reading a turnout figure has no way to tell a measured value from a
          // typed one, so a zeroed or invented figure is worse than an absent one:
          // it is a claim about the count of people who have voted.
          const EelStatusChip(label: unavailable, severity: EelSeverity.warning),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.list, color: EelColors.of(context).primaryMuted, size: 32),
            title: const Text('Queue Position', style: EelTextStyle.titleSmall),
            subtitle: const Text(queueUnavailable, style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          const EelButton(
            label: 'Refresh Queue',
            icon: Icons.refresh,
            // Disabled, not wired: no queue endpoint or client exists, and the figures this screen renders are literals.
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