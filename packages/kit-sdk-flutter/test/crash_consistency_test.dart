import 'package:flutter_test/flutter_test.dart';
import 'package:eel_kit_sdk/src/offline_queue.dart';

/// Crash-simulating storage that throws after [crashAfterNWrites] saves.
class CrashingStorage implements OutboxStorage {
  CrashingStorage({
    required this.crashAfterNWrites,
    InMemoryOutboxStorage? delegate,
  }) : _delegate = delegate ?? InMemoryOutboxStorage();

  final int crashAfterNWrites;
  final InMemoryOutboxStorage _delegate;
  int _writes = 0;

  @override
  Future<void> save(OutboxEntry entry) async {
    _writes++;
    if (_writes > crashAfterNWrites) throw StateError('simulated crash');
    return _delegate.save(entry);
  }

  @override
  Future<void> remove(String id) => _delegate.remove(id);

  @override
  Future<List<OutboxEntry>> loadAll() => _delegate.loadAll();
}

void main() {
  group('crash consistency', () {
    test(
        'crash after journal write leaves no partial entry after recovery',
        () async {
      final delegate = InMemoryOutboxStorage();
      final crashingStorage =
          CrashingStorage(crashAfterNWrites: 1, delegate: delegate);
      final queue = EelOfflineQueue(storage: crashingStorage);
      await queue.init();

      try {
        await queue.enqueue(
            id: 'c-1', idempotencyKey: 'idem-c-1', payload: {'x': 1});
      } catch (_) {
        // Expected crash on phase 2.
      }

      final recoveryQueue = EelOfflineQueue(storage: delegate);
      await recoveryQueue.init();

      for (final e in recoveryQueue.all()) {
        expect(e.stepCommitted, isTrue,
            reason: 'recovered entries must be fully committed');
      }
    });

    test('fully committed entry survives recovery', () async {
      final storage = InMemoryOutboxStorage();
      final queue = EelOfflineQueue(storage: storage);
      await queue.init();

      await queue.enqueue(
          id: 'c-2', idempotencyKey: 'idem-c-2', payload: {'y': 2});

      final recoveryQueue = EelOfflineQueue(storage: storage);
      await recoveryQueue.init();

      expect(recoveryQueue.all().length, 1);
      expect(recoveryQueue.all().first.id, 'c-2');
    });
  });
}
