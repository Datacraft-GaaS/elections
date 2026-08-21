import 'package:flutter_test/flutter_test.dart';
import 'package:sqflite_common_ffi/sqflite_ffi.dart';

import 'package:eel_lms_offline/features/sync/offline_queue.dart';

void main() {
  setUpAll(() {
    sqfliteFfiInit();
    databaseFactory = databaseFactoryFfi;
  });

  test('pending items found after offline save', () async {
    final queue = OfflineQueue();

    final entry = OfflineQueueEntry(
      id: 'sync-test-id-1',
      workerId: 'worker-sync-001',
      moduleId: 'module-sync-001',
      idempotencyKey: 'sync-idem-key-001',
      answers: {'q1': 'a1'},
      status: 'pending',
      createdAt: DateTime.now(),
    );

    await queue.insert(entry);

    final pending = await queue.getPending();
    expect(
      pending.any((e) => e.idempotencyKey == 'sync-idem-key-001'),
      isTrue,
      reason: 'Saved entry must appear in pending list',
    );
  });

  test('sync clears pending on success', () async {
    final queue = OfflineQueue();

    final entry = OfflineQueueEntry(
      id: 'sync-test-id-2',
      workerId: 'worker-sync-002',
      moduleId: 'module-sync-002',
      idempotencyKey: 'sync-idem-key-002',
      answers: {'q1': 'a1'},
      status: 'pending',
      createdAt: DateTime.now(),
    );

    await queue.insert(entry);
    await queue.markSynced('sync-idem-key-002');

    final pending = await queue.getPending();
    expect(
      pending.any((e) => e.idempotencyKey == 'sync-idem-key-002'),
      isFalse,
      reason: 'Synced entry must not appear in pending list',
    );
  });
}
