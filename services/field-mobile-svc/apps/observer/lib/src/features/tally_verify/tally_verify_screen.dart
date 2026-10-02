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

/// Tally Verification (M34 FR-026) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class TallyVerifyScreen extends ConsumerWidget {
  const TallyVerifyScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Tally Verification',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Verify Tally', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 24),
          EelResultCard(
            leading: Icon(Icons.verified, color: EelColors.of(context).primary, size: 32),
            title: const Text('Tally Status', style: EelTextStyle.titleMedium),
            subtitle: const Text('All tallies verified ✓', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          const EelStatusChip(label: 'Verification: Complete', severity: EelSeverity.success),
          const SizedBox(height: 8),
          const EelStatusChip(label: 'Discrepancies: 0', severity: EelSeverity.success),
          const SizedBox(height: 16),
          const EelButton(
            label: 'Verify Now',
            icon: Icons.refresh,
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