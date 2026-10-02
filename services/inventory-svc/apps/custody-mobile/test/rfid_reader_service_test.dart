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

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:eel_custody_mobile/src/services/rfid_reader_service.dart';

/// In-memory [RfidReaderTransport] used to drive vendor services offline.
class _FakeTransport implements RfidReaderTransport {
  final controller = StreamController<RfidTransportEvent>.broadcast();
  final connectedIds = <String>[];
  var disposeCount = 0;

  void emit(RfidTransportEvent event) => controller.add(event);

  @override
  Stream<RfidTransportEvent> get events => controller.stream;

  @override
  Future<void> discover() async {}

  @override
  Future<void> connect(String deviceId) async {
    connectedIds.add(deviceId);
    emit(RfidConnectedEvent(deviceId));
  }

  @override
  Future<void> disconnect() async {
    emit(const RfidDisconnectedEvent());
  }

  @override
  Future<void> dispose() async {
    disposeCount++;
    await controller.close();
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

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
      await Future<void>.delayed(Duration.zero);
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

  group('TransportDrivenReaderService', () {
    late _FakeTransport transport;
    late ZebraRfd40ReaderService svc;

    setUp(() {
      transport = _FakeTransport();
      svc = ZebraRfd40ReaderService(transport: transport);
    });

    tearDown(() async {
      await svc.dispose();
    });

    test('connectToDevice forwards device id and reaches connected', () async {
      final states = <ReaderConnectionState>[];
      svc.connectionStateStream.listen(states.add);
      await svc.connectToDevice('RFD40-001');
      await Future<void>.delayed(Duration.zero);
      expect(transport.connectedIds, ['RFD40-001']);
      expect(svc.connectionState, ReaderConnectionState.connected);
      expect(states, containsAll(
          [ReaderConnectionState.connecting, ReaderConnectionState.connected]));
    });

    test('tag read event maps into RfidScanResult', () async {
      final completer = Completer<RfidScanResult>();
      await svc.connectToDevice('RFD40-001');
      svc.scanResults.listen(completer.complete);
      transport.emit(const RfidTagReadEvent(
        epc: 'e28011702000010a02bc3f2a',
        tamperBit: true,
        challengeVerified: true,
        rssi: -48.5,
      ));
      final result = await completer.future.timeout(const Duration(seconds: 2));
      // EPCs are normalized to uppercase hex.
      expect(result.epc, 'E28011702000010A02BC3F2A');
      expect(result.tamperIndicator, isTrue);
      expect(result.challengeValid, isTrue);
      expect(result.rssi, -48.5);
    });

    test('empty-EPC tag reads are dropped', () async {
      var received = false;
      await svc.connectToDevice('RFD40-001');
      svc.scanResults.listen((_) => received = true);
      transport.emit(const RfidTagReadEvent(
        epc: '',
        tamperBit: false,
        challengeVerified: false,
      ));
      await Future<void>.delayed(Duration.zero);
      expect(received, isFalse);
    });

    test('transport error event moves state to error', () async {
      await svc.connectToDevice('RFD40-001');
      transport.emit(const RfidTransportErrorEvent('link lost'));
      await Future<void>.delayed(Duration.zero);
      expect(svc.connectionState, ReaderConnectionState.error);
    });

    test('disconnect event from hardware moves state to disconnected',
        () async {
      await svc.connectToDevice('RFD40-001');
      transport.emit(const RfidDisconnectedEvent());
      await Future<void>.delayed(Duration.zero);
      expect(svc.connectionState, ReaderConnectionState.disconnected);
    });

    test('dispose releases the transport exactly once', () async {
      await svc.dispose();
      expect(transport.disposeCount, 1);
      expect(() => svc.startScan(), throwsStateError);
    });
  });

  group('PlatformChannelReaderTransport missing native bridge', () {
    test('connect without a registered handler surfaces error state',
        () async {
      final svc = ZebraRfd40ReaderService();
      await svc.connectToDevice('RFD40-001');
      expect(svc.connectionState, ReaderConnectionState.error);
      await svc.dispose();
    });

    test('startScan without a registered handler surfaces error state',
        () async {
      final svc = CipherlabRs36ReaderService();
      await svc.startScan();
      expect(svc.connectionState, ReaderConnectionState.error);
      await svc.dispose();
    });
  });

  group('createReaderService factory', () {
    test('synthetic model returns SyntheticReaderService', () {
      final svc = createReaderService(RfidReaderModel.synthetic);
      expect(svc, isA<SyntheticReaderService>());
      svc.dispose();
    });

    test('zebraRfd40 model returns ZebraRfd40ReaderService', () {
      final svc = createReaderService(RfidReaderModel.zebraRfd40);
      expect(svc, isA<ZebraRfd40ReaderService>());
      svc.dispose();
    });

    test('cipherlabRs36 model returns CipherlabRs36ReaderService', () {
      final svc = createReaderService(RfidReaderModel.cipherlabRs36);
      expect(svc, isA<CipherlabRs36ReaderService>());
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
