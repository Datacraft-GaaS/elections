/// Diaspora Kit Station Screen — Beautiful, ergonomic, accessible.
///
/// Station selection is profile-scoped: choosing an import profile changes the
/// working timezone, so any station picked under the previous profile is
/// cleared rather than silently carried over.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

import 'models.dart';
import 'providers.dart';

class StationScreen extends ConsumerWidget {
  const StationScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final profile = ref.watch(importProfileProvider);
    final stations = ref.watch(stationsProvider);
    final selected = ref.watch(selectedStationProvider);

    return EelScaffold(
      title: 'Diaspora Station',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: Column(
        children: [
          const Text('Station Overview', style: EelTextStyle.titleLarge),
          const SizedBox(height: 16),

          Text('Import profile', style: EelTextStyle.titleSmall),
          const SizedBox(height: 8),
          DropdownButtonFormField<ImportProfile>(
            key: const Key('profile-dropdown'),
            initialValue: profile,
            decoration: const InputDecoration(
              labelText: 'Import profile',
              border: OutlineInputBorder(),
            ),
            items: [
              for (final p in ImportProfile.values)
                DropdownMenuItem(value: p, child: Text(p.label)),
            ],
            onChanged: (next) {
              if (next == null) return;
              // A new profile means a new timezone, so the station chosen
              // under the old one no longer applies.
              ref.read(importProfileProvider.notifier).state = next;
              ref.read(selectedStationProvider.notifier).state = null;
            },
          ),
          const SizedBox(height: 16),

          Text('Embassy station', style: EelTextStyle.titleSmall),
          const SizedBox(height: 8),
          DropdownButtonFormField<EmbassyStation>(
            key: const Key('station-dropdown'),
            initialValue: selected,
            decoration: const InputDecoration(
              labelText: 'Embassy station',
              border: OutlineInputBorder(),
            ),
            items: [
              for (final s in stations)
                DropdownMenuItem(value: s, child: Text('$s')),
            ],
            onChanged: (next) =>
                ref.read(selectedStationProvider.notifier).state = next,
          ),
          const SizedBox(height: 16),

          if (selected != null) ...[
            EelResultCard(
              leading: Icon(Icons.location_on,
                  color: EelColors.of(context).primary, size: 32),
              title: Text(selected.stationCode, style: EelTextStyle.titleMedium),
              subtitle: Text(
                '${selected.city} • ${selected.timezone} • '
                '${selected.isOpen ? 'Open' : 'Closed'}',
                style: EelTextStyle.bodyMedium,
              ),
            ),
            const SizedBox(height: 16),
            EelButton(
              key: const Key('open-session-btn'),
              label: 'Open Session',
              icon: Icons.play_arrow,
              fullWidth: true,
              // Disabled, not wired: no station-session client exists, so no session can be opened.
              //
              // This was an empty callback, which renders a live control that does
              // nothing when tapped. EelButton treats a null callback as disabled,
              // so the unavailable state is now visible rather than silent.
              onPressed: null,
            ),
            const SizedBox(height: 16),
          ],

          const EelStatusChip(
              label: 'Connection: Online', severity: EelSeverity.success),
          const SizedBox(height: 8),
          const EelStatusChip(
              label: 'Forms synced: Yes', severity: EelSeverity.success),
        ],
      ),
    );
  }
}
