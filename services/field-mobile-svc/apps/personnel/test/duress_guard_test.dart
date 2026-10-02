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

import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:eel_personnel/src/services/duress_guard.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('duress pin enqueues a covert alert; normal pin does not', () async {
    final queue = EelOfflineQueue(storage: InMemoryOutboxStorage());
    await queue.init();
    final guard = DuressGuard(queue: queue, duressPin: '000000');

    expect(await guard.check('123456'), isFalse);
    expect(queue.pending, isEmpty);

    expect(await guard.check('000000'), isTrue);
    expect(queue.pending, hasLength(1));
    expect(queue.pending.single.payload['type'], 'KitDuressAlert');
  });
}
