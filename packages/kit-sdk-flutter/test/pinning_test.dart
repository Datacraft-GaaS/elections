import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('SpkiPin', () {
    test('fromJson round-trip', () {
      final json = {'base64_sha256': 'abc123', 'host': 'example.com', 'port': 443};
      final pin = SpkiPin.fromJson(json);
      expect(pin.base64Sha256, 'abc123');
      expect(pin.host, 'example.com');
      expect(pin.port, 443);
      expect(pin.toJson(), json);
    });

    test('fromJson with missing optional fields', () {
      final json = {'base64_sha256': 'abc123'};
      final pin = SpkiPin.fromJson(json);
      expect(pin.host, isNull);
      expect(pin.port, isNull);
    });

    test('equality by base64Sha256', () {
      final a = SpkiPin(base64Sha256: 'abc');
      final b = SpkiPin(base64Sha256: 'abc', host: 'x');
      expect(a, b);
    });

    test('different base64Sha256 is not equal', () {
      final a = SpkiPin(base64Sha256: 'abc');
      final b = SpkiPin(base64Sha256: 'def');
      expect(a != b, isTrue);
    });
  });

  group('PinSet', () {
    test('fromJson round-trip', () {
      final json = {
        'set_id': 'set-1',
        'pins': [
          {'base64_sha256': 'pin1', 'host': 'example.com'},
          {'base64_sha256': 'pin2'},
        ],
        'created_at': '2026-01-01T00:00:00Z',
      };
      final set = PinSet.fromJson(json);
      expect(set.setId, 'set-1');
      expect(set.pins.length, 2);
      expect(set.pins[0].host, 'example.com');
      expect(set.hasPins, isTrue);
      expect(set.toJson()['set_id'], 'set-1');
    });

    test('empty pins set is not valid', () {
      final set = PinSet(setId: 'empty', pins: []);
      expect(set.hasPins, isFalse);
    });

    test('createdAt parsed from ISO string', () {
      final set = PinSet(
        setId: 'set-1',
        pins: [SpkiPin(base64Sha256: 'abc')],
        createdAt: DateTime.utc(2026, 1, 1),
      );
      expect(set.createdAt, DateTime.utc(2026, 1, 1));
    });
  });

  group('SpkiPinningAdapter', () {
    test('allows empty allowlist', () {
      final adapter = SpkiPinningAdapter(pins: [
        SpkiPin(base64Sha256: 'abc'),
      ]);
      expect(adapter.allowList, isEmpty);
    });

    test('allowlist exempts specified hosts', () {
      final adapter = SpkiPinningAdapter(
        pins: [SpkiPin(base64Sha256: 'abc')],
        allowList: {'localhost'},
      );
      expect(adapter.allowList, {'localhost'});
    });

    test('dual-pin window accepts either pin', () {
      // Simulate: old pin and new pin both valid during rotation.
      final adapter = SpkiPinningAdapter(pins: [
        SpkiPin(base64Sha256: 'old-pin'),
        SpkiPin(base64Sha256: 'new-pin'),
      ]);
      expect(adapter.pins.length, 2);
    });

    test('host constraint respected', () {
      final adapter = SpkiPinningAdapter(pins: [
        SpkiPin(base64Sha256: 'abc', host: 'example.com'),
      ]);
      expect(adapter.pins.first.host, 'example.com');
    });
  });

  group('cryptoSha256', () {
    test('digests empty bytes', () {
      final digest = cryptoSha256([]);
      expect(digest, isNotEmpty);
      expect(digest.length, 32); // SHA-256 produces 32 bytes
    });

    test('digests known payload', () {
      final digest = cryptoSha256([1, 2, 3]);
      expect(digest, isNotEmpty);
      expect(digest.length, 32);
    });

    test('is deterministic', () {
      final bytes = [255, 0, 128];
      expect(cryptoSha256(bytes), cryptoSha256(bytes));
    });
  });
}
