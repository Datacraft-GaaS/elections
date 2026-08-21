import 'package:flutter_test/flutter_test.dart';
import 'package:eel_kit_sdk/src/offline_queue.dart';

void main() {
  group('InMemoryOutboxStorage', () {
    test('save and load round-trip', () async {
      final storage = InMemoryOutboxStorage();
      final entry = OutboxEntry(
        id: 'test-1',
        idempotencyKey: 'idem-1',
        payload: {'key': 'value'},
        createdAt: DateTime.now(),
        stepCommitted: true,
      );
      await storage.save(entry);
      final loaded = await storage.loadAll();
      expect(loaded.length, 1);
      expect(loaded.first.id, 'test-1');
    });
  });
}
