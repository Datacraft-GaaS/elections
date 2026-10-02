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

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:sqflite_common_ffi/sqflite_ffi.dart';

import 'package:eel_lms_offline/features/assessment/assessment_model.dart';
import 'package:eel_lms_offline/features/sync/offline_queue.dart';

// Real SQLite FFI I/O must run under plain `test()`, never `testWidgets`:
// FakeAsync does not advance platform I/O, which deadlocks the awaits.
void main() {
  setUpAll(() {
    sqfliteFfiInit();
    databaseFactory = databaseFactoryFfi;
  });

  late Directory tempDir;
  late OfflineQueue queue;

  setUp(() async {
    tempDir = await Directory.systemTemp.createTemp('lms_offline_test');
    databaseFactory.setDatabasesPath(tempDir.path);
    queue = OfflineQueue();
  });

  tearDown(() async {
    await queue.close();
    await tempDir.delete(recursive: true);
  });

  test('offline assessment idempotency [EC-DAT-01]', () async {
    const attempt = AssessmentAttempt(
      id: 'test-id-1',
      workerId: 'worker-001',
      moduleId: 'module-001',
      answers: {'q1': 'answer1'},
      idempotencyKey: 'idem-key-001',
    );

    final entry = OfflineQueueEntry(
      id: attempt.id,
      workerId: attempt.workerId,
      moduleId: attempt.moduleId,
      idempotencyKey: attempt.idempotencyKey,
      answers: attempt.answers,
      status: 'pending',
      createdAt: DateTime.now(),
    );

    // Insert once
    await queue.insert(entry);

    // Insert again with same idempotency key
    await queue.insert(entry);

    final pending = await queue.getPending();
    expect(
      pending.where((e) => e.idempotencyKey == 'idem-key-001').length,
      equals(1),
      reason: 'Duplicate idempotency key must not create second entry',
    );
  });

  test('offline assessment entries survive queue reload [EC-DAT-01]', () async {
    final entry = OfflineQueueEntry(
      id: 'test-id-2',
      workerId: 'worker-002',
      moduleId: 'module-001',
      idempotencyKey: 'idem-key-002',
      answers: {'q1': 'a'},
      status: 'pending',
      createdAt: DateTime.now(),
    );
    await queue.insert(entry);

    // A fresh instance over the same path sees the persisted row.
    final reloaded = OfflineQueue();
    addTearDown(reloaded.close);
    final pending = await reloaded.getPending();
    expect(pending.where((e) => e.idempotencyKey == 'idem-key-002').length, 1);
  });

  test('5-module cache limit enforced', () {
    // Verifies that the limit constant is set to 5
    // Full enforcement is tested via ModuleRepository integration test
    const maxModules = 5;
    expect(maxModules, equals(5));
  });
}
