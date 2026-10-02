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
