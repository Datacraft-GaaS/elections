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

/// Incident Report (M29 FR-C7) — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class IncidentReportScreen extends ConsumerWidget {
  const IncidentReportScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Report Incident',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      // EelScaffold's scrollable variant already provides the viewport, so a
      // ListView here is a scrollable inside a scrollable and throws on
      // performResize. A Column is what the variant expects.
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Anonymous Incident Report',
              style: EelTextStyle.titleLarge),
          const SizedBox(height: 8),
          const EelBanner(
            message:
                'Your report is anonymous. No identity is linked to this submission.',
            severity: EelSeverity.info,
          ),
          const SizedBox(height: 16),
          EelTextField(
            label: 'Describe the incident',
            maxLines: 4,
            hintText: 'What happened? Where?',
          ),
          const SizedBox(height: 16),
          EelChips(
            options: const [
              ChipOption(label: 'Intimidation'),
              ChipOption(label: 'Bribery'),
              ChipOption(label: 'Turned Away'),
              ChipOption(label: 'Other'),
            ],
            onSelected: (chip) {},
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Submit Anonymously',
            icon: Icons.send,
            fullWidth: true,
            onPressed: () => ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(
                  content: Text('Report queued (anonymous, GPS optional)')),
            ),
          ),
        ],
      ),
    );
  }
}
