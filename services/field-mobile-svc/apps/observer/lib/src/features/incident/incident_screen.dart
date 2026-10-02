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

/// Observer Incident (M34 FR-028) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class IncidentScreen extends ConsumerWidget {
  const IncidentScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Report Incident',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(24),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Report Incident', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 24),
          const EelTextField(
            label: 'Incident Description',
            maxLines: 4,
            hintText: 'Describe what happened',
          ),
          const SizedBox(height: 16),
          const EelTextField(
            label: 'Category',
            hintText: 'Intimidation, Bribery, Turned Away, Other',
          ),
          const SizedBox(height: 16),
          const EelButton(
            label: 'Submit Incident',
            icon: Icons.report,
            fullWidth: true,
            // Disabled, not wired: this screen has no domain client for the call it describes.
            //
            // This was an empty callback, which renders a live control that does
            // nothing when tapped. EelButton treats a null callback as disabled,
            // so the unavailable state is now visible rather than silent.
            onPressed: null,
          ),
          const SizedBox(height: 12),
          EelButton(
            label: 'Cancel',
            variant: EelButtonVariant.outlined,
            fullWidth: true,
            onPressed: () => Navigator.pop(context),
          ),
        ],
      ),
    );
  }
}