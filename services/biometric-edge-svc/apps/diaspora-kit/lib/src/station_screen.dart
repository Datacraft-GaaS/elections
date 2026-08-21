/// Home screen: embassy station selection and session management.

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'models.dart';
import 'providers.dart';
import 'form34a_screen.dart';
import 'pouch_screen.dart';
import 'tz_display.dart';

class StationScreen extends ConsumerWidget {
  const StationScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final stations = ref.watch(stationsProvider);
    final selected = ref.watch(selectedStationProvider);
    final profile = ref.watch(importProfileProvider);
    final openStations = stations.where((s) => s.isOpen).toList();

    return Scaffold(
      appBar: AppBar(
        title: const Text('eel Diaspora Kit'),
        actions: [
          IconButton(
            icon: const Icon(Icons.inventory_2),
            tooltip: 'Pouches',
            onPressed: () => Navigator.push(
              context,
              MaterialPageRoute(builder: (_) => const PouchScreen()),
            ),
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Station picker
            Text('Embassy Station', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 8),
            DropdownButtonFormField<EmbassyStation>(
              key: const Key('station-dropdown'),
              initialValue: selected,
              hint: const Text('Select open station'),
              items: openStations
                  .map((s) => DropdownMenuItem(value: s, child: Text(s.toString())))
                  .toList(),
              onChanged: (s) {
                ref.read(selectedStationProvider.notifier).state = s;
                // Clear cached embassy list when profile or station changes.
                if (s != null) {
                  ref.read(stationsProvider.notifier).state =
                      ref.read(stationsProvider);
                }
              },
            ),
            const SizedBox(height: 16),

            // Import profile toggle
            Text('Import Profile', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 8),
            DropdownButtonFormField<ImportProfile>(
              key: const Key('profile-dropdown'),
              initialValue: profile,
              items: ImportProfile.values
                  .map(
                    (p) => DropdownMenuItem(value: p, child: Text(p.label)),
                  )
                  .toList(),
              onChanged: (p) {
                if (p != null) {
                  ref.read(importProfileProvider.notifier).state = p;
                  // Profile switch clears cached embassy list.
                  ref.read(selectedStationProvider.notifier).state = null;
                }
              },
            ),
            const SizedBox(height: 16),

            // TZ display
            if (selected != null) ...[
              TzDisplayWidget(stationTimezone: selected.timezone),
              const SizedBox(height: 16),
            ],

            // Open session button
            if (selected != null)
              FilledButton.icon(
                key: const Key('open-session-btn'),
                icon: const Icon(Icons.how_to_vote),
                label: const Text('Open Session / Capture Form 34A'),
                onPressed: () => Navigator.push(
                  context,
                  MaterialPageRoute(
                    builder: (_) => Form34AScreen(station: selected),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}
