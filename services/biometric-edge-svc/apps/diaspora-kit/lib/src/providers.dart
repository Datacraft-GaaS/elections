import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'models.dart';

// ---- Embassy stations cache -----------------------------------------------

/// Cached list of open embassy stations from local storage.
/// In production this would be populated from the Isar DB by eel_kit_sdk.
final stationsProvider = StateProvider<List<EmbassyStation>>((ref) {
  // Demo seed data; replace with Isar query in production.
  return [
    const EmbassyStation(
      id: 'station-001',
      countryIso: 'US',
      city: 'Washington',
      timezone: 'America/New_York',
      stationCode: 'US-DC-001',
      isOpen: true,
    ),
    const EmbassyStation(
      id: 'station-002',
      countryIso: 'GB',
      city: 'London',
      timezone: 'Europe/London',
      stationCode: 'UK-LON-001',
      isOpen: true,
    ),
    const EmbassyStation(
      id: 'station-003',
      countryIso: 'AE',
      city: 'Dubai',
      timezone: 'Asia/Dubai',
      stationCode: 'AE-DXB-001',
      isOpen: false,
    ),
  ];
});

// ---- Selected station -------------------------------------------------------

final selectedStationProvider = StateProvider<EmbassyStation?>((ref) => null);

// ---- Import profile ---------------------------------------------------------

final importProfileProvider = StateProvider<ImportProfile>(
  (ref) => ImportProfile.restOfWorld,
);

// ---- Offline form queue -----------------------------------------------------

final offlineFormQueueProvider =
    StateNotifierProvider<OfflineFormQueueNotifier, List<Form34AEntry>>(
  (ref) => OfflineFormQueueNotifier(),
);

class OfflineFormQueueNotifier extends StateNotifier<List<Form34AEntry>> {
  OfflineFormQueueNotifier() : super([]);

  void enqueue(Form34AEntry form) {
    state = [...state, form];
  }

  void removeById(String formId) {
    state = state.where((f) => f.formId != formId).toList();
  }
}

// ---- Pouches ----------------------------------------------------------------

final pouchesProvider =
    StateNotifierProvider<PouchNotifier, List<PouchRecord>>(
  (ref) => PouchNotifier(),
);

class PouchNotifier extends StateNotifier<List<PouchRecord>> {
  PouchNotifier() : super([]);

  void add(PouchRecord pouch) {
    state = [...state, pouch];
  }

  void addTransfer(String pouchId, CustodyTransfer transfer) {
    state = [
      for (final p in state)
        if (p.id == pouchId)
          PouchRecord(
            id: p.id,
            formIds: p.formIds,
            sealId: p.sealId,
            sealedAt: p.sealedAt,
            transfers: [...p.transfers, transfer],
          )
        else
          p,
    ];
  }
}
