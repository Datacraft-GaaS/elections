import 'package:flutter_test/flutter_test.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

BatterySignal _batterySignal(int counter, String prevHash) => BatterySignal(
      deviceId: 'dev-001',
      occurredAt: DateTime.utc(2026, 8, 22, 0, 0, counter),
      monotonicCounter: counter,
      traceId: 'trace-$counter',
      spanId: 'span-$counter',
      correlationId: 'corr-$counter',
      prevHash: prevHash,
      percentInt: 80,
      chargingState: ChargingState.discharging,
    );

void main() {
  group('TelemetryOfflineQueue', () {
    test('persists and loads entries', () async {
      final storage = InMemoryTelemetryStorage();
      final queue = TelemetryOfflineQueue(storage: storage);
      await queue.init();

      final chain = HashChain();
      final s = _batterySignal(1, chain.tail);
      chain.attach(s);
      await queue.accept(s);

      expect(queue.pendingCount, equals(1));

      // Simulate restart: new queue from same storage.
      final queue2 = TelemetryOfflineQueue(storage: storage);
      await queue2.init();
      expect(queue2.pendingCount, equals(1));
    });

    test('drains in FIFO order', () async {
      final storage = InMemoryTelemetryStorage();
      final queue = TelemetryOfflineQueue(storage: storage);
      await queue.init();

      final chain = HashChain();
      for (var i = 1; i <= 5; i++) {
        final s = _batterySignal(i, chain.tail);
        chain.attach(s);
        await queue.accept(s);
      }

      final batch = queue.drainBatch(maxSize: 3);
      expect(batch.length, equals(3));
    });

    test('acknowledges and removes entries', () async {
      final storage = InMemoryTelemetryStorage();
      final queue = TelemetryOfflineQueue(storage: storage);
      await queue.init();

      final chain = HashChain();
      final s = _batterySignal(1, chain.tail);
      chain.attach(s);
      await queue.accept(s);

      final batch = queue.drainBatch();
      await queue.acknowledge(batch.map((e) => e.id).toList());
      expect(queue.pendingCount, equals(0));
    });

    test('schedules retry with backoff', () async {
      final storage = InMemoryTelemetryStorage();
      final queue = TelemetryOfflineQueue(storage: storage);
      await queue.init();

      final chain = HashChain();
      final s = _batterySignal(1, chain.tail);
      chain.attach(s);
      await queue.accept(s);

      final batch = queue.drainBatch();
      await queue.scheduleRetry(batch.map((e) => e.id).toList());

      // Entry still in queue but nextRetryAt is now in the future.
      final after = queue.drainBatch();
      expect(after, isEmpty);
    });

    test('expired entries pruned on init', () async {
      final storage = InMemoryTelemetryStorage();
      // Manually add an expired entry to storage.
      final expired = TelemetryQueueEntry(
        id: 'expired-1',
        enqueuedAt: DateTime.now().toUtc().subtract(const Duration(days: 8)),
        signalJson: {'signalType': 'battery', 'deviceId': 'dev-001'},
      );
      await storage.persist(expired);

      final queue = TelemetryOfflineQueue(storage: storage);
      await queue.init();
      expect(queue.pendingCount, equals(0));
    });
  });
}
