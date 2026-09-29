import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:timezone/data/latest.dart' as tz_data;

import 'package:eel_diaspora_kit/src/models.dart';
import 'package:eel_diaspora_kit/src/offline_queue_diaspora.dart';
import 'package:eel_diaspora_kit/src/providers.dart';
import 'package:eel_diaspora_kit/src/station_screen.dart';
import 'package:eel_diaspora_kit/src/form34a_screen.dart';

void main() {
  setUpAll(() {
    tz_data.initializeTimeZones();
  });
  // ---- Offline queue unit tests -------------------------------------------

  group('OfflineQueueDiaspora', () {
    test('enqueued form is retrieved when ready', () {
      final queue = OfflineQueueDiaspora();
      final form = Form34AEntry(
        stationId: 'station-001',
        formId: 'FORM-001',
        results: {'A': 10},
        rejected: 0,
        disputed: 0,
      );
      final releaseAt = DateTime.utc(2027, 8, 10, 12, 0, 0);
      queue.enqueue(form, releaseAt);

      final after = DateTime.utc(2027, 8, 10, 13, 0, 0);
      final ready = queue.readyForms(after);
      expect(ready.length, equals(1));
      expect(ready.first.form.formId, equals('FORM-001'));
    });

    test('form not ready before deferred time', () {
      final queue = OfflineQueueDiaspora();
      final form = Form34AEntry(
        stationId: 'station-001',
        formId: 'FORM-002',
        results: {'B': 5},
        rejected: 0,
        disputed: 0,
      );
      final releaseAt = DateTime.utc(2027, 8, 10, 18, 0, 0);
      queue.enqueue(form, releaseAt);

      final before = DateTime.utc(2027, 8, 10, 12, 0, 0);
      expect(queue.readyForms(before), isEmpty);
    });

    test('remove deletes form from queue', () {
      final queue = OfflineQueueDiaspora();
      final form = Form34AEntry(
        stationId: 's',
        formId: 'FORM-DEL',
        results: {},
        rejected: 0,
        disputed: 0,
      );
      queue.enqueue(form, DateTime.utc(2027, 1, 1));
      queue.remove('FORM-DEL');
      expect(queue.length, equals(0));
    });

    test('deferred form not in queue persists correctly', () {
      final queue = OfflineQueueDiaspora();
      expect(queue.length, equals(0));
      queue.enqueue(
        Form34AEntry(
          stationId: 's',
          formId: 'F1',
          results: {'X': 1},
          rejected: 0,
          disputed: 0,
        ),
        DateTime.utc(2027, 8, 10),
      );
      queue.enqueue(
        Form34AEntry(
          stationId: 's',
          formId: 'F2',
          results: {'Y': 2},
          rejected: 0,
          disputed: 0,
        ),
        DateTime.utc(2027, 8, 11),
      );
      expect(queue.length, equals(2));
    });
  });

  // ---- Widget tests -------------------------------------------------------

  group('StationScreen', () {
    testWidgets('profile switch clears station selection', (tester) async {
      await tester.pumpWidget(
        const ProviderScope(
          child: MaterialApp(home: StationScreen()),
        ),
      );
      await tester.pumpAndSettle();

      // Find the profile dropdown and switch profile.
      final profileDropdown = find.byKey(const Key('profile-dropdown'));
      expect(profileDropdown, findsOneWidget);

      // Tap to open the dropdown.
      await tester.tap(profileDropdown);
      await tester.pumpAndSettle();

      // Select 'London' profile.
      final londonOption = find.text('London').last;
      await tester.tap(londonOption);
      await tester.pumpAndSettle();

      // Station selection should be cleared (no Open Session button).
      expect(find.byKey(const Key('open-session-btn')), findsNothing);
    });

    testWidgets('station selection shows TZ display and session button',
        (tester) async {
      await tester.pumpWidget(
        const ProviderScope(
          child: MaterialApp(home: StationScreen()),
        ),
      );
      await tester.pumpAndSettle();

      // Open station dropdown and pick first option.
      await tester.tap(find.byKey(const Key('station-dropdown')));
      await tester.pumpAndSettle();

      final stationOption = find.text('US-DC-001 - Washington (US)').last;
      await tester.tap(stationOption);
      await tester.pumpAndSettle();

      // Should now show the open session button.
      expect(find.byKey(const Key('open-session-btn')), findsOneWidget);
    });
  });

  group('Form34AScreen', () {
    testWidgets('captured form is added to offline queue', (tester) async {
      const station = EmbassyStation(
        id: 'station-001',
        countryIso: 'US',
        city: 'Washington',
        timezone: 'America/New_York',
        stationCode: 'US-DC-001',
        isOpen: true,
      );

      late WidgetRef capturedRef;

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            formSignerProvider.overrideWithValue(
              (message) async => 'test-signature:$message',
            ),
          ],
          child: MaterialApp(
            home: Consumer(
              builder: (context, ref, _) {
                capturedRef = ref;
                return Form34aScreen(station: station);
              },
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Fill tallies with default values (already 0), submit.
      await tester.tap(find.byKey(const Key('submit-form34a')));
      await tester.pumpAndSettle();

      final queue = capturedRef.read(offlineFormQueueProvider);
      expect(queue.length, equals(1));
      expect(queue.first.stationId, equals('station-001'));
      expect(queue.first.hsmSignature, isNotNull);
    });
  });
}
