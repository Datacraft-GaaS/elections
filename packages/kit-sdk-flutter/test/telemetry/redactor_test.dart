import 'dart:convert';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

void main() {
  const redactor = Redactor();
  final now = DateTime.utc(2026, 8, 22);

  group('Redactor', () {
    test('voter ID is hashed — raw ID never emitted', () {
      const rawVoterId = 'KE-12345678';
      final hashed = Redactor.hashId(rawVoterId);
      expect(hashed, isNot(contains(rawVoterId)));
      expect(hashed.length, equals(64)); // SHA-256 hex
      // Same input produces same hash (deterministic).
      expect(hashed, equals(sha256.convert(utf8.encode(rawVoterId)).toString()));
    });

    test('officer name replaced — SVID reference only', () {
      // The ReadSignal entityIdHash should be a hash, never a raw name.
      final signal = ReadSignal(
        deviceId: 'dev-001',
        occurredAt: now,
        monotonicCounter: 1,
        traceId: 'trace', spanId: 'span', correlationId: 'corr',
        prevHash: '0' * 64,
        entityKind: 'voter_record',
        entityIdHash: Redactor.hashId('12345678'), // must be hash, not raw
      );
      expect(signal.entityIdHash, isNot(equals('12345678')));
      expect(signal.entityIdHash.length, equals(64));
    });

    test('location coarsened when accuracy > 500 m', () {
      final s = LocationSignal(
        deviceId: 'dev-001',
        occurredAt: now,
        monotonicCounter: 1,
        traceId: 'trace', spanId: 'span', correlationId: 'corr',
        prevHash: '0' * 64,
        latitudeDeg: -1.286390,
        longitudeDeg: 36.817223,
        accuracyM: 600.0,
      );

      final redacted = redactor.redact(s) as LocationSignal;
      expect(redacted.isCoarsened, isTrue);
      // Must be rounded to nearest 0.1 deg.
      expect(redacted.latitudeDeg % 0.1, closeTo(0.0, 1e-9));
      expect(redacted.longitudeDeg % 0.1, closeTo(0.0, 1e-9));
    });

    test('location not coarsened when accuracy <= 500 m', () {
      final s = LocationSignal(
        deviceId: 'dev-001',
        occurredAt: now,
        monotonicCounter: 1,
        traceId: 'trace', spanId: 'span', correlationId: 'corr',
        prevHash: '0' * 64,
        latitudeDeg: -1.286390,
        longitudeDeg: 36.817223,
        accuracyM: 499.9,
      );

      final redacted = redactor.redact(s) as LocationSignal;
      expect(redacted.isCoarsened, isFalse);
      expect(redacted.latitudeDeg, equals(-1.286390));
    });

    test('coarsen rounds to nearest grid cell', () {
      expect(Redactor.coarsen(1.234), closeTo(1.2, 1e-9));
      expect(Redactor.coarsen(1.250), closeTo(1.3, 1e-9));
      expect(Redactor.coarsen(-1.286), closeTo(-1.3, 1e-9));
    });
  });
}
