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
