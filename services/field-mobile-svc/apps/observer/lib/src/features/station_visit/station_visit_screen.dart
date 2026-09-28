/// Station Visit (M34 FR-024) — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class StationVisitScreen extends ConsumerWidget {
  const StationVisitScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'Station Visit',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Record Station Visit', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 24),
          const EelTextField(
            label: 'Station Code',
            hintText: 'Enter station code',
          ),
          const SizedBox(height: 16),
          const EelTextField(
            label: 'Arrival Time',
            hintText: 'Select time',
          ),
          const SizedBox(height: 16),
          const EelTextField(
            label: 'Notes',
            maxLines: 3,
            hintText: 'Add visit notes',
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Submit Visit',
            icon: Icons.check,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}