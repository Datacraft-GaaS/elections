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

/// Station Finder (M13 FR-C4) — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class StationFinderScreen extends ConsumerWidget {
  const StationFinderScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Find Station',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      // EelScaffold's scrollable variant already provides the viewport, so a
      // ListView here is a scrollable inside a scrollable and throws on
      // performResize. A Column is what the variant expects.
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Polling Station Finder', style: EelTextStyle.titleLarge),
          const SizedBox(height: 8),
          Text(
            'Search by station code or ward name. Works offline once map is downloaded.',
            style: EelTextStyle.bodyMedium,
          ),
          const SizedBox(height: 16),
          EelTextField(
            label: 'Search stations',
            hintText: 'Enter station code or ward',
          ),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.location_on, color: EelColors.of(context).primary, size: 32),
            title: Text('KE-NBI-001', style: EelTextStyle.titleMedium),
            subtitle: Text('Westlands • Open 06:00 - 18:00', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 12),
          EelResultCard(
            leading: Icon(Icons.location_on, color: EelColors.of(context).primary, size: 32),
            title: Text('KE-NBI-002', style: EelTextStyle.titleMedium),
            subtitle: Text('Karen • Open 06:00 - 18:00', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelBanner(
            message: 'Offline map available — previously downloaded regions are accessible.',
            severity: EelSeverity.info,
          ),
        ],
      ),
    );
  }
}
