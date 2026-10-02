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

/// Bookmarks (M29 FR-C8) — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class BookmarksScreen extends ConsumerWidget {
  const BookmarksScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Bookmarks',
      variant: EelScaffoldVariant.fixedAppBar,
      padding: const EdgeInsets.all(16),
      body: ListView(
        children: [
          const Text('Saved Bookmarks', style: EelTextStyle.titleLarge),
          const SizedBox(height: 16),
          EelResultCard(
            leading: Icon(Icons.bookmark, color: EelColors.of(context).primary, size: 32),
            title: Text('Station KE-NBI-001', style: EelTextStyle.titleMedium),
            subtitle: Text('Added: 2026-09-01', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 12),
          EelResultCard(
            leading: Icon(Icons.bookmark, color: EelColors.of(context).primary, size: 32),
            title: Text('KE-NBI-002 Results', style: EelTextStyle.titleMedium),
            subtitle: Text('Added: 2026-08-28', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelBanner(
            message: 'Bookmarks are stored locally on your device only. No data leaves your phone.',
            severity: EelSeverity.info,
          ),
        ],
      ),
    );
  }
}
