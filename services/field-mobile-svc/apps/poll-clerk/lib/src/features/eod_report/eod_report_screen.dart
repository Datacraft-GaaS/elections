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

/// EOD Report (M28 FR-007) — Elegant, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class EODReportScreen extends ConsumerWidget {
  const EODReportScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'EOD Report',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      // EelScaffold's scrollable variant already provides the viewport, so a
      // ListView here is a scrollable inside a scrollable and throws on
      // performResize. A Column is what the variant expects.
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('End of Day Report', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.check_circle, color: EelColors.of(context).primary, size: 32),
            title: const Text('Daily Summary', style: EelTextStyle.titleMedium),
          // Was a literal. These figures are read as counts of voters and as a
          // measure of kit availability; an official has no way to tell a measured
          // value from a typed one, so an absent figure is the honest state.
            subtitle: const Text(
                'Unavailable — no tally source is wired into this app.',
                style: EelTextStyle.bodyMedium,
            ),
          ),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.analytics, color: EelColors.of(context).primaryMuted, size: 32),
            title: const Text('Kiosk Status', style: EelTextStyle.titleMedium),
            // Was `Operational - 100% uptime`, a literal.
            subtitle: const Text(
                'Unavailable — no health source is wired into this app.',
                style: EelTextStyle.bodyMedium,
            ),
          ),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.analytics, color: EelColors.of(context).primaryMuted, size: 32),
            title: const Text('EOD Metrics', style: EelTextStyle.titleMedium),
            subtitle: const Text('Voter turnout trend', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.list_alt, color: EelColors.of(context).primaryMuted, size: 32),
            title: const Text('Report Summary', style: EelTextStyle.titleMedium),
          // Was a literal. These figures are read as counts of voters and as a
          // measure of kit availability; an official has no way to tell a measured
          // value from a typed one, so an absent figure is the honest state.
            subtitle: const Text(
                'Unavailable — no tally source is wired into this app.',
                style: EelTextStyle.bodyMedium,
            ),
          ),
          const SizedBox(height: 16),
          const EelButton(
            label: 'Download PDF',
            icon: Icons.download,
            fullWidth: true,
            // Disabled, not wired: no end-of-day report endpoint or client exists to produce a PDF from.
            //
            // This was an empty callback, which renders a live control that does
            // nothing when tapped. EelButton treats a null callback as disabled,
            // so the unavailable state is now visible rather than silent.
            onPressed: null,
          // Disabled, not wired: no end-of-day report endpoint or client exists to produce a PDF from.
          //
          // This was an empty callback, which renders a live control that does
          // nothing when tapped. EelButton treats a null callback as disabled,
          // so the unavailable state is now visible rather than silent.
          ),
          const SizedBox(height: 16),
          const EelButton(
            label: 'View Full Report',
            fullWidth: true,
            // Disabled, not wired: no end-of-day report endpoint or client exists to
            // render a full report from.
            onPressed: null,
          ),
        ],
      ),
    );
  }
}