import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/src/screens/rfid_scan.dart';
import '../lib/src/services/rfid_reader_service.dart';

Widget _wrap(Widget child) => MaterialApp(home: child);

void main() {
  late SyntheticReaderService reader;

  setUp(() {
    reader = SyntheticReaderService();
  });

  tearDown(() async {
    await reader.dispose();
  });

  group('RfidScanScreen', () {
    testWidgets('renders Connect button when disconnected', (tester) async {
      await tester.pumpWidget(_wrap(RfidScanScreen(
        officerId: 'OFF-001',
        readerService: reader,
      )));
      expect(find.text('Connect RFID reader'), findsOneWidget);
    });

    testWidgets('shows waiting message before first scan', (tester) async {
      await tester.pumpWidget(_wrap(RfidScanScreen(
        officerId: 'OFF-001',
        readerService: reader,
      )));
      expect(find.text('Waiting for tag scan...'), findsOneWidget);
    });

    testWidgets('scan result card appears after inject', (tester) async {
      await tester.pumpWidget(_wrap(RfidScanScreen(
        officerId: 'OFF-001',
        readerService: reader,
      )));

      reader.injectScan(RfidScanResult(
        epc: 'E28011702000010A02BC3F2A',
        readAt: DateTime(2026, 6, 1),
        tamperIndicator: false,
        challengeValid: true,
      ));

      await tester.pump();
      expect(find.text('E28011702000010A02BC3F2A'), findsWidgets);
      expect(find.text('Tag intact'), findsOneWidget);
    });

    testWidgets('tamper indicator shows TAMPER DETECTED', (tester) async {
      await tester.pumpWidget(_wrap(RfidScanScreen(
        officerId: 'OFF-001',
        readerService: reader,
      )));

      reader.injectTamperRead('DEADBEEF00000001');
      await tester.pump();

      expect(find.text('TAMPER DETECTED'), findsOneWidget);
    });

    testWidgets('Submit button disabled until checkbox confirmed', (tester) async {
      await tester.pumpWidget(_wrap(RfidScanScreen(
        officerId: 'OFF-001',
        readerService: reader,
      )));

      reader.injectScan(RfidScanResult(
        epc: 'ABC123',
        readAt: DateTime.now(),
        tamperIndicator: false,
        challengeValid: true,
      ));
      await tester.pump();

      final submitBtn = find.text('Submit state transition');
      expect(submitBtn, findsOneWidget);
      // Button should be disabled (no confirm yet)
      final elevated = tester.widget<ElevatedButton>(
        find.widgetWithText(ElevatedButton, 'Submit state transition'),
      );
      expect(elevated.onPressed, isNull);
    });

    testWidgets('Submit enabled after checkbox ticked', (tester) async {
      await tester.pumpWidget(_wrap(RfidScanScreen(
        officerId: 'OFF-001',
        readerService: reader,
      )));

      reader.injectScan(RfidScanResult(
        epc: 'ABC123',
        readAt: DateTime.now(),
        tamperIndicator: false,
        challengeValid: true,
      ));
      await tester.pump();

      await tester.tap(find.byType(CheckboxListTile));
      await tester.pump();

      final elevated = tester.widget<ElevatedButton>(
        find.widgetWithText(ElevatedButton, 'Submit state transition'),
      );
      expect(elevated.onPressed, isNotNull);
    });

    testWidgets('onTransitionSubmit callback invoked on submit', (tester) async {
      String? submittedEpc;
      String? submittedOfficer;

      await tester.pumpWidget(_wrap(RfidScanScreen(
        officerId: 'OFF-001',
        readerService: reader,
        onTransitionSubmit: (epc, officer) async {
          submittedEpc = epc;
          submittedOfficer = officer;
        },
      )));

      reader.injectScan(RfidScanResult(
        epc: 'ABC123',
        readAt: DateTime.now(),
        tamperIndicator: false,
        challengeValid: true,
      ));
      await tester.pump();
      await tester.tap(find.byType(CheckboxListTile));
      await tester.pump();
      await tester.tap(find.text('Submit state transition'));
      await tester.pumpAndSettle();

      expect(submittedEpc, 'ABC123');
      expect(submittedOfficer, 'OFF-001');
    });

    testWidgets('offline queue counter shown after scan', (tester) async {
      await tester.pumpWidget(_wrap(RfidScanScreen(
        officerId: 'OFF-001',
        readerService: reader,
      )));

      reader.injectScan(RfidScanResult(
        epc: 'XYZ',
        readAt: DateTime.now(),
        tamperIndicator: false,
        challengeValid: true,
      ));
      await tester.pump();

      expect(find.textContaining('queued offline'), findsOneWidget);
    });

    testWidgets('duress button is present', (tester) async {
      await tester.pumpWidget(_wrap(RfidScanScreen(
        officerId: 'OFF-001',
        readerService: reader,
      )));
      expect(find.text('Hold for duress alert'), findsOneWidget);
    });
  });
}
