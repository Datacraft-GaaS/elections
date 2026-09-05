/// Reference Library (M28 FR-011) — Elegant, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class ReferenceLibraryScreen extends ConsumerWidget {
  const ReferenceLibraryScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Reference Library',
      variant: EelScaffoldVariant.fixedAppBar,
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          const EelBanner(
            message: 'Quick access to election procedures, forms, and legal references.',
            severity: 'info',
          ),
          const SizedBox(height: 16),
          const Text('Categories', style: EelTextStyle.titleMedium),
          const SizedBox(height: 8),
          EelChips(
            options: const [
              'Procedures',
              'Forms',
              'Legal References',
              'Training Materials',
              'Incident Guides'
            ],
            onSelected: (chip) {},
          ),
          const SizedBox(height: 24),
          EelResultCard(
            leading: Icon(Icons.book, color: EelColors.of(context).primary, size: 32),
            title: Text('Polling Station Procedures (v2.1)', style: EelTextStyle.titleMedium),
            subtitle: Text('Updated: 2026-08-15 • 42 pages', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 12),
          EelResultCard(
            leading: Icon(Icons.description, color: EelColors.of(context).primaryMuted, size: 32),
            title: Text('Voter Registration Form (FRM-007)', style: EelTextStyle.titleMedium),
            subtitle: Text('Current version • Available in 11 languages', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 12),
          EelButton(
            label: 'Download Selected',
            icon: Icons.download,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}