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
