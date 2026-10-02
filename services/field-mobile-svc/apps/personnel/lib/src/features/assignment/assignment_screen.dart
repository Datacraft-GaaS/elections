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

/// Personnel Assignment (M29 FR-015) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class AssignmentScreen extends ConsumerWidget {
  const AssignmentScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Assignment',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('My Assignment', style: EelTextStyle.titleLarge),
          const SizedBox(height: 24),
          EelResultCard(
            leading: Icon(Icons.assignment, color: EelColors.of(context).primary, size: 32),
            title: const Text('Current Assignment', style: EelTextStyle.titleMedium),
            subtitle: const Text('Station KE-NBI-001 • Poll Clerk', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          const EelStatusChip(label: 'Status: Active', severity: EelSeverity.success),
          const SizedBox(height: 8),
          const EelStatusChip(label: 'Shift: 06:00 - 18:00', severity: EelSeverity.info),
        ],
      ),
    );
  }
}
