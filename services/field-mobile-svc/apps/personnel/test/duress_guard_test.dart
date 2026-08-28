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
