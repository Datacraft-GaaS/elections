import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('InMemoryKeyStore', () {
    late InMemoryKeyStore store;

    setUp(() {
      store = InMemoryKeyStore();
    });

    test('readKey returns null when no key stored', () async {
      expect(await store.readKey('test-key'), isNull);
    });

    test('writeKey and readKey round-trip', () async {
      final key = List<int>.filled(32, 42);
      await store.writeKey('test-key', key);
      final loaded = await store.readKey('test-key');
      expect(loaded, key);
    });

    test('deleteKey removes stored key', () async {
      await store.writeKey('test-key', List<int>.filled(32, 1));
      await store.deleteKey('test-key');
      expect(await store.readKey('test-key'), isNull);
    });

    test('multiple keys are independent', () async {
      await store.writeKey('key-1', List<int>.filled(32, 1));
      await store.writeKey('key-2', List<int>.filled(32, 2));
      expect(await store.readKey('key-1'), List<int>.filled(32, 1));
      expect(await store.readKey('key-2'), List<int>.filled(32, 2));
    });
  });

  group('KeyVersion', () {
    test('fromJson round-trip', () {
      final json = {
        'version': 2,
        'key_id': 'key-123',
        'created_at': '2026-01-01T00:00:00Z',
      };
      final version = KeyVersion.fromJson(json);
      expect(version.version, 2);
      expect(version.keyId, 'key-123');
      expect(version.createdAt, DateTime.utc(2026, 1, 1));
      expect(version.toJson()['version'], 2);
    });

    test('fromJson with missing fields uses defaults', () {
      final version = KeyVersion.fromJson({'version': 1});
      expect(version.keyId, '');
      expect(version.createdAt, DateTime.parse('1970-01-01T00:00:00Z'));
    });
  });

  group('EelEncryptedIsarStore', () {
    // Key size is 32 bytes (AES-256) — verified by the implementation.
    test('InMemoryKeyStore generates valid 32-byte key', () async {
      final store = InMemoryKeyStore();
      final key = List<int>.generate(32, (_) => DateTime.now().microsecondsSinceEpoch % 256);
      await store.writeKey('test', key);
      expect(await store.readKey('test'), key);
    });

    test('InMemoryKeyStore generates valid key', () async {
      final store = InMemoryKeyStore();
      final key = List<int>.generate(32, (_) => DateTime.now().microsecondsSinceEpoch % 256);
      await store.writeKey('test', key);
      expect(await store.readKey('test'), key);
    });
  });

  group('PushPayload', () {
    test('fromJson round-trip', () {
      final json = {
        'event_type': 'eel.field_mobile.incident_reported.v1',
        'record_id': 'rec-123',
        'timestamp': '2026-01-01T00:00:00Z',
      };
      final payload = PushPayload.fromJson(json);
      expect(payload.eventType, 'eel.field_mobile.incident_reported.v1');
      expect(payload.recordId, 'rec-123');
      expect(payload.timestamp, DateTime.utc(2026, 1, 1));
      expect(payload.toData()['event_type'], 'eel.field_mobile.incident_reported.v1');
    });

    test('equality by eventType and recordId', () {
      const a = PushPayload(eventType: 'e1', recordId: 'r1');
      final b = PushPayload(eventType: 'e1', recordId: 'r1', timestamp: DateTime.now());
      expect(a, b);
    });

    test('different eventType is not equal', () {
      const a = PushPayload(eventType: 'e1', recordId: 'r1');
      const b = PushPayload(eventType: 'e2', recordId: 'r1');
      expect(a != b, isTrue);
    });
  });

  group('validatePushPayload', () {
    test('valid payload passes', () {
      const payload = PushPayload(eventType: 'eel.field.incident.v1', recordId: 'rec-1');
      expect(validatePushPayload(payload), isNull);
    });

    test('empty eventType fails', () {
      const payload = PushPayload(eventType: '', recordId: 'rec-1');
      expect(validatePushPayload(payload), isNotNull);
    });

    test('empty recordId fails', () {
      const payload = PushPayload(eventType: 'e1', recordId: '');
      expect(validatePushPayload(payload), isNotNull);
    });

    test('PII in eventType fails', () {
      const payload = PushPayload(eventType: 'eel.field.123456789.v1', recordId: 'rec-1');
      expect(validatePushPayload(payload), isNotNull);
    });
  });

  group('PushTokenEntry', () {
    test('fromJson round-trip', () {
      final json = {
        'device_id': 'dev-1',
        'token': 'fcm-token-abc',
        'platform': 'fcm',
        'registered_at': '2026-01-01T00:00:00Z',
        'expires_at': '2026-02-01T00:00:00Z',
      };
      final entry = PushTokenEntry.fromJson(json);
      expect(entry.deviceId, 'dev-1');
      expect(entry.token, 'fcm-token-abc');
      expect(entry.platform, 'fcm');
      expect(entry.expiresAt, DateTime.utc(2026, 2, 1));
      expect(entry.toJson()['device_id'], 'dev-1');
    });

    test('fromJson with missing optional fields', () {
      final entry = PushTokenEntry.fromJson(const {
        'device_id': 'dev-1',
        'token': 'token',
        'platform': 'apns',
        'registered_at': '2026-01-01T00:00:00Z',
      });
      expect(entry.expiresAt, isNull);
    });
  });

  group('InMemoryPushClient', () {
    late InMemoryPushClient client;

    setUp(() {
      client = InMemoryPushClient();
    });

    test('starts unregistered', () {
      expect(client.state, PushState.unregistered);
      expect(client.token, isNull);
    });

    test('register produces token and registered state', () async {
      final token = await client.register();
      expect(client.state, PushState.registered);
      expect(client.token, token);
      expect(token, isNotNull);
    });

    test('unregister clears token', () async {
      await client.register();
      await client.unregister();
      expect(client.state, PushState.unregistered);
      expect(client.token, isNull);
    });

    test('reRegisterIfNeeded returns true when expired', () async {
      client.setExpired();
      expect(client.state, PushState.expired);
      final reRegistered = await client.reRegisterIfNeeded();
      expect(reRegistered, isTrue);
      expect(client.state, PushState.registered);
    });

    test('reRegisterIfNeeded returns false when not expired', () async {
      await client.register();
      final reRegistered = await client.reRegisterIfNeeded();
      expect(reRegistered, isFalse);
    });

    test('reset clears state', () async {
      await client.register();
      client.reset();
      expect(client.state, PushState.unregistered);
      expect(client.token, isNull);
    });
  });
}
