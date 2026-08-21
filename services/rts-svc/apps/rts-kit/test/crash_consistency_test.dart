/// Crash-consistency property test for the RTS queue.
///
/// Simulates a kill mid-transmit by throwing mid-way through the two-phase
/// enqueue commit, then reinitialising the queue and asserting the partially
/// written entry is purged.
///
/// Also verifies the successful path: a fully committed entry survives
/// reinitialisation and is present in [EelOfflineQueue.pending].
library;

import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter_test/flutter_test.dart';

// ---- Fault-injecting storage ------------------------------------------------

/// Storage that throws after the first [save] call to simulate a crash between
/// the two-phase commit's journal write and the commit write.
class CrashAfterFirstSaveStorage implements OutboxStorage {
  CrashAfterFirstSaveStorage(this._inner);
  final InMemoryOutboxStorage _inner;
  int _saveCount = 0;

  @override
  Future<void> save(OutboxEntry entry) async {
    _saveCount++;
    if (_saveCount == 1) {
      // Phase 1 (journal write) succeeds.
      await _inner.save(entry);
      return;
    }
    // Phase 2 (commit write) crashes.
    throw StateError('simulated power cut after phase-1 write');
  }

  @override
  Future<void> remove(String id) => _inner.remove(id);

  @override
  Future<List<OutboxEntry>> loadAll() => _inner.loadAll();
}

// ---- Tests ------------------------------------------------------------------

void main() {
  group('EelOfflineQueue crash-consistency', () {
    test('partial write (stepCommitted=false) is purged on reinit', () async {
      final inner = InMemoryOutboxStorage();
      final crashStorage = CrashAfterFirstSaveStorage(inner);
      final queue = EelOfflineQueue(storage: crashStorage);
      await queue.init();

      // Attempt enqueue: phase-1 succeeds, phase-2 throws.
      await expectLater(
        () => queue.enqueue(
          id: 'crash-test-id',
          idempotencyKey: 'ik-crash',
          payload: {'form_type': 'form34a'},
        ),
        throwsA(isA<StateError>()),
      );

      // The entry exists in storage with stepCommitted=false.
      final saved = await inner.loadAll();
      expect(saved, hasLength(1));
      expect(saved.first.stepCommitted, isFalse);

      // Reinit on a new queue using the same storage: partial entry removed.
      final queue2 = EelOfflineQueue(storage: inner);
      await queue2.init();

      expect(queue2.pending, isEmpty);
      expect(queue2.all(), isEmpty);
      final afterReinit = await inner.loadAll();
      expect(afterReinit, isEmpty);
    });

    test('committed entry survives reinit and appears in pending', () async {
      final storage = InMemoryOutboxStorage();
      final queue = EelOfflineQueue(storage: storage);
      await queue.init();

      await queue.enqueue(
        id: 'stable-id',
        idempotencyKey: 'ik-stable',
        payload: {'form_type': 'form34a', 'test': true},
      );

      expect(queue.pending, hasLength(1));

      // Reinit.
      final queue2 = EelOfflineQueue(storage: storage);
      await queue2.init();

      expect(queue2.pending, hasLength(1));
      expect(queue2.pending.first.id, equals('stable-id'));
      expect(queue2.pending.first.stepCommitted, isTrue);
    });

    test('expired entries are removed on reinit', () async {
      final storage = InMemoryOutboxStorage();

      // Manually insert an entry with createdAt > 7 days ago.
      final expired = OutboxEntry(
        id: 'expired-id',
        idempotencyKey: 'ik-expired',
        payload: {},
        createdAt: DateTime.now().subtract(const Duration(days: 8)),
        stepCommitted: true,
      );
      await storage.save(expired);

      final queue = EelOfflineQueue(storage: storage);
      await queue.init();

      expect(queue.all(), isEmpty);
      // Also purged from storage.
      final rows = await storage.loadAll();
      expect(rows, isEmpty);
    });

    test('markState transitions are reflected in pending list', () async {
      final storage = InMemoryOutboxStorage();
      final queue = EelOfflineQueue(storage: storage);
      await queue.init();

      final entry = await queue.enqueue(
        id: 'state-id',
        idempotencyKey: 'ik-state',
        payload: {},
      );

      expect(queue.pending, hasLength(1));

      await queue.markState(entry.id, OutboxEntryState.syncing);
      expect(queue.pending, isEmpty);

      await queue.markState(entry.id, OutboxEntryState.synced);
      expect(queue.pending, isEmpty);
    });
  });
}
