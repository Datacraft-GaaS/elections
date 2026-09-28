import 'package:flutter_test/flutter_test.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

// Build a minimal DropSignal for chain testing.
DropSignal _signal(int counter, String prevHash) => DropSignal(
      deviceId: 'dev-001',
      occurredAt: DateTime.utc(2026, 8, 22),
      monotonicCounter: counter,
      traceId: 'trace-$counter',
      spanId: 'span-$counter',
      correlationId: 'corr-$counter',
      prevHash: prevHash,
      peakAccelG: 4.0,
      windowMs: 200,
      severity: SignalSeverity.critical,
    );

void main() {
  group('HashChain integrity', () {
    test('attaches hashes correctly for 100 signals', () {
      final chain = HashChain();
      final signals = <DropSignal>[];

      for (var i = 0; i < 100; i++) {
        final s = _signal(i, chain.tail);
        chain.attach(s);
        signals.add(s);
      }

      expect(HashChain.verify(signals), isTrue);
    });

    test('verifies 10 000-signal sequence', () {
      final chain = HashChain();
      final signals = <DropSignal>[];

      for (var i = 0; i < 10000; i++) {
        final s = _signal(i, chain.tail);
        chain.attach(s);
        signals.add(s);
      }

      expect(HashChain.verify(signals), isTrue);
    });

    test('detects tampered entry', () {
      final chain = HashChain();
      final signals = <DropSignal>[];

      for (var i = 0; i < 50; i++) {
        final s = _signal(i, chain.tail);
        chain.attach(s);
        signals.add(s);
      }

      // Tamper: corrupt thisHash of signal 25.
      signals[25].thisHash = 'deadbeef0000000000000000000000000000000000000000000000000000cafe';

      expect(HashChain.verify(signals), isFalse);
    });

    test('tail advances after each attach', () {
      final chain = HashChain();
      final initialTail = chain.tail;

      final s = _signal(0, chain.tail);
      chain.attach(s);

      expect(chain.tail, isNot(equals(initialTail)));
      expect(chain.tail, equals(s.thisHash));
    });

    test('first prevHash matches initial hash when firstPrevHash supplied', () {
      final chain = HashChain(initialHash: 'a' * 64);
      final s = _signal(0, chain.tail);
      chain.attach(s);

      expect(HashChain.verify([s], firstPrevHash: 'a' * 64), isTrue);
    });
  });
}
