import 'package:flutter_test/flutter_test.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

void main() {
  group('EelOfflineQueue', () {
    test('enqueue creates committed entry', () async {
      final storage = InMemoryOutboxStorage();
      final queue = EelOfflineQueue(storage: storage);
      await queue.init();

      final entry = await queue.enqueue(
        id: 'q-1',
        idempotencyKey: 'idem-q-1',
        payload: {'event': 'vote'},
      );

      expect(entry.stepCommitted, isTrue);
      expect(queue.pending.length, 1);
    });

    test('markState transitions work', () async {
      final storage = InMemoryOutboxStorage();
      final queue = EelOfflineQueue(storage: storage);
      await queue.init();

      await queue.enqueue(id: 'q-2', idempotencyKey: 'idem-q-2', payload: {});
      await queue.markState('q-2', OutboxEntryState.synced);

      expect(queue.pending, isEmpty);
    });
  });
}
