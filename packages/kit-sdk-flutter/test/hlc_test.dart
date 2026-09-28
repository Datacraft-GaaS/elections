import 'package:flutter_test/flutter_test.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

void main() {
  group('Hlc', () {
    test('parse and format round-trip', () {
      const s = '1700000000000.0.1';
      final hlc = Hlc.parse(s);
      expect(hlc.toString(), s);
    });

    test('tick advances counter within same ms', () {
      const h = Hlc(ms: 100, counter: 5, node: 1);
      final ticked = h.tick(100);
      expect(ticked.ms, 100);
      expect(ticked.counter, 6);
    });

    test('tick advances ms when wall clock is ahead', () {
      const h = Hlc(ms: 100, counter: 5, node: 1);
      final ticked = h.tick(200);
      expect(ticked.ms, 200);
      expect(ticked.counter, 0);
    });

    test('merge takes max ms', () {
      const a = Hlc(ms: 100, counter: 0, node: 1);
      const b = Hlc(ms: 200, counter: 0, node: 2);
      final merged = a.merge(b, 50);
      expect(merged.ms, 200);
    });

    test('compareTo is total order', () {
      const a = Hlc(ms: 100, counter: 0, node: 1);
      const b = Hlc(ms: 100, counter: 1, node: 1);
      expect(a.compareTo(b), lessThan(0));
    });
  });
}
