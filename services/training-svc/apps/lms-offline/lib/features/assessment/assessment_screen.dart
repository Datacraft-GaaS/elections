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

/// LMS Offline Assessment — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class AssessmentScreen extends ConsumerWidget {
  const AssessmentScreen({super.key, this.moduleId});

  final String? moduleId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Assessment',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      // EelScaffold's scrollable variant already provides the viewport, so a
      // ListView here is a scrollable inside a scrollable and throws on
      // performResize. A Column is what the variant expects.
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Module Assessment', style: EelTextStyle.titleLarge),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.quiz,
                color: EelColors.of(context).primary, size: 32),
            title: const Text('Question 1 of 10', style: EelTextStyle.titleMedium),
            subtitle:
                const Text('Select the best answer', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelChips(
            options: const [
              ChipOption(label: 'A'),
              ChipOption(label: 'B'),
              ChipOption(label: 'C'),
              ChipOption(label: 'D'),
            ],
            onSelected: (chip) {},
          ),
          const SizedBox(height: 24),
          const EelButton(
            label: 'Next Question',
            icon: Icons.arrow_forward,
            fullWidth: true,
            // Disabled, not wired: the assessment has no Question model, so there is nothing to advance to.
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
