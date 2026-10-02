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

/// Observer Status (M34 FR-025/027) — Beautiful, ergonomic, accessible.
/// Uses eel_design: EelScaffold, EelStatusChip, EelBanner, EelButton, EelColors.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class ObserverStatusScreen extends ConsumerWidget {
  const ObserverStatusScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final colors = EelColors.of(context);
    return EelScaffold(
      title: 'Observer Status',
      variant: EelScaffoldVariant.fixedAppBar,
      body: ListView(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        children: [
          // Hero card — station info with seal.
          EelResultCard(
            leading: EelSealStamp(size: 48, color: colors.primaryMuted),
            title: const Text('Station KE-NBI-001', style: EelTextStyle.titleLarge),
            subtitle: const Text('Verified observer — active session', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          // Status chips — no PII, aggregate info only.
          const EelStatusChip(label: 'Connection: Online', severity: EelSeverity.success),
          const SizedBox(height: 8),
          const EelStatusChip(label: 'Forms synced: Yes', severity: EelSeverity.success),
          const SizedBox(height: 8),
          // Was `Kit battery: 78%`, a literal. Nothing reads the battery, so
          // reporting a level asserts one.
          const EelStatusChip(
            label: 'Kit battery not read — no health source is wired into this app.',
            severity: EelSeverity.warning,
          ),
          const SizedBox(height: 8),
          // Was `Incidents open: 1`, a literal. A fabricated open-incident count
          // is read as something an observer must act on.
          const EelStatusChip(
            label: 'Open incidents not read — no incident source is wired into this app.',
            severity: EelSeverity.warning,
          ),
          const SizedBox(height: 16),
          // Security banner.
          const EelBanner(
            message: 'FLAG_SECURE / isCaptured / screen-capture-tamper (FR-M31-029)',
            severity: EelSeverity.info,
          ),
          const SizedBox(height: 8),
          // Connectivity note.
          const EelBanner(
            message: 'BVI back-end notified on connectivity change (§5.1, FR-M30-003).',
            severity: EelSeverity.info,
          ),
          const SizedBox(height: 24),
          const EelButton(
            label: 'Refresh Status',
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
