import 'dart:async';

import 'package:flutter_test/flutter_test.dart';

import '../lib/src/services/rfid_reader_service.dart';

void main() {
  group('SyntheticReaderService', () {
    late SyntheticReaderService svc;

    setUp(() {
      svc = SyntheticReaderService();
    });

    tearDown(() async {
      await svc.dispose();
    });

    test('initial state is disconnected', () {
      expect(svc.connectionState, ReaderConnectionState.disconnected);
    });

    test('startScan transitions to scanning', () async {
      final states = <ReaderConnectionState>[];
      svc.connectionStateStream.listen(states.add);
      await svc.startScan();
      expect(states, contains(ReaderConnectionState.scanning));
    });

    test('connectToDevice transitions connected', () async {
      final states = <ReaderConnectionState>[];
      svc.connectionStateStream.listen(states.add);
      await svc.connectToDevice('dev-001');
      expect(svc.connectionState, ReaderConnectionState.connected);
      expect(states, contains(ReaderConnectionState.connected));
    });

    test('injectScan emits a scan result', () async {
      final completer = Completer<RfidScanResult>();
      svc.scanResults.listen(completer.complete);
      final result = RfidScanResult(
        epc: 'E28011702000010A02BC3F2A',
        readAt: DateTime.now(),
        tamperIndicator: false,
        challengeValid: true,
      );
      svc.injectScan(result);
      final received = await completer.future.timeout(const Duration(seconds: 2));
      expect(received.epc, 'E28011702000010A02BC3F2A');
      expect(received.challengeValid, isTrue);
    });

    test('injectTamperRead emits result with tamperIndicator=true', () async {
      final completer = Completer<RfidScanResult>();
      svc.scanResults.listen(completer.complete);
      svc.injectTamperRead('DEADBEEF00000001');
      final received = await completer.future.timeout(const Duration(seconds: 2));
      expect(received.tamperIndicator, isTrue);
      expect(received.challengeValid, isFalse);
    });

    test('disconnect transitions to disconnected', () async {
      await svc.connectToDevice('dev-001');
      await svc.disconnect();
      expect(svc.connectionState, ReaderConnectionState.disconnected);
    });
  });

  group('createReaderService factory', () {
    test('synthetic model returns SyntheticReaderService', () {
      final svc = createReaderService(RfidReaderModel.synthetic);
      expect(svc, isA<SyntheticReaderService>());
      svc.dispose();
    });

    test('zebraRfd40 model returns a service (currently synthetic stub)', () {
      final svc = createReaderService(RfidReaderModel.zebraRfd40);
      expect(svc, isNotNull);
      svc.dispose();
    });

    test('cipherlabRs36 model returns a service (currently synthetic stub)', () {
      final svc = createReaderService(RfidReaderModel.cipherlabRs36);
      expect(svc, isNotNull);
      svc.dispose();
    });
  });

  group('RfidScanResult', () {
    test('toString includes key fields', () {
      final r = RfidScanResult(
        epc: 'ABCD',
        readAt: DateTime(2026),
        tamperIndicator: true,
        challengeValid: false,
      );
      expect(r.toString(), contains('ABCD'));
      expect(r.toString(), contains('tamper=true'));
    });
  });
}
