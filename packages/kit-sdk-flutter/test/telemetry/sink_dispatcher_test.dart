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

import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

BatterySignal _signal(int counter, String prevHash) => BatterySignal(
      deviceId: 'dev-001',
      occurredAt: DateTime.utc(2026, 8, 22),
      monotonicCounter: counter,
      traceId: 'trace-$counter',
      spanId: 'span-$counter',
      correlationId: 'corr-$counter',
      prevHash: prevHash,
      percentInt: 80,
      chargingState: ChargingState.discharging,
    );

class _FakeAdapter implements HttpClientAdapter {
  _FakeAdapter({required this.statusCode});
  final int statusCode;
  int callCount = 0;

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<List<int>>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    callCount++;
    return ResponseBody.fromString(
      jsonEncode({'accepted': [], 'rejected': []}),
      statusCode,
      headers: {
        Headers.contentTypeHeader: [Headers.jsonContentType],
      },
    );
  }

  @override
  void close({bool force = false}) {}
}

void main() {
  group('SinkDispatcher', () {
    test('happy path: batch dispatched and queue drained', () async {
      final storage = InMemoryTelemetryStorage();
      final queue = TelemetryOfflineQueue(storage: storage);
      await queue.init();

      final chain = HashChain();
      for (var i = 0; i < 5; i++) {
        final s = _signal(i, chain.tail);
        chain.attach(s);
        await queue.accept(s);
      }
      expect(queue.pendingCount, equals(5));

      final adapter = _FakeAdapter(statusCode: 200);
      final dio = Dio()..httpClientAdapter = adapter;

      final dispatcher = SinkDispatcher(
        queue: queue,
        http: dio,
        ingestBaseUrl: 'https://test.internal',
        deviceSvid: 'dev-001',
        batchSize: 10,
      );

      await dispatcher.triggerNow();

      expect(adapter.callCount, equals(1));
      expect(queue.pendingCount, equals(0));
    });

    test('network failure: entries scheduled for retry', () async {
      final storage = InMemoryTelemetryStorage();
      final queue = TelemetryOfflineQueue(storage: storage);
      await queue.init();

      final chain = HashChain();
      for (var i = 0; i < 3; i++) {
        final s = _signal(i, chain.tail);
        chain.attach(s);
        await queue.accept(s);
      }

      final adapter = _FakeAdapter(statusCode: 500);
      final dio = Dio()..httpClientAdapter = adapter;

      final dispatcher = SinkDispatcher(
        queue: queue,
        http: dio,
        ingestBaseUrl: 'https://test.internal',
        deviceSvid: 'dev-001',
      );

      await dispatcher.triggerNow();

      // Entries remain in queue (scheduled for retry).
      expect(queue.pendingCount, equals(3));
    });

    test('batch size limit respected', () async {
      final storage = InMemoryTelemetryStorage();
      final queue = TelemetryOfflineQueue(storage: storage);
      await queue.init();

      final chain = HashChain();
      for (var i = 0; i < 50; i++) {
        final s = _signal(i, chain.tail);
        chain.attach(s);
        await queue.accept(s);
      }

      final adapter = _FakeAdapter(statusCode: 200);
      final dio = Dio()..httpClientAdapter = adapter;

      final dispatcher = SinkDispatcher(
        queue: queue,
        http: dio,
        ingestBaseUrl: 'https://test.internal',
        deviceSvid: 'dev-001',
        batchSize: 20,
      );

      await dispatcher.triggerNow();

      // 20 dispatched, 30 remain.
      expect(queue.pendingCount, equals(30));
    });
  });
}
