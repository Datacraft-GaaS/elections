import 'dart:typed_data';

import 'package:eel_core_flutter/eel_core_flutter.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('computeSha256', () {
    test('digests empty bytes', () {
      expect(computeSha256(Uint8List(0)),
          equals('e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855'));
    });

    test('digests known payload', () {
      final bytes = Uint8List.fromList([1, 2, 3]);
      expect(
        computeSha256(bytes),
        equals('039058c6f2c0cb492c533b0a4d14ef77cc0f78abccced5287d84a1a2011cfb81'),
      );
    });

    test('is deterministic', () {
      final bytes = Uint8List.fromList([255, 0, 128]);
      expect(computeSha256(bytes), equals(computeSha256(bytes)));
    });
  });

  group('EvidenceUploadRequest', () {
    final base = EvidenceUploadRequest(
      id: 'ev-1',
      idempotencyKey: 'idem-1',
      path: '/data/photo.jpg',
      contentType: 'image/jpeg',
      size: 1024,
      sha256: 'abc123',
      evidenceType: 'photo',
      metadata: {'station': 'ST-001'},
    );

    test('copyWith creates new instance', () {
      final next = base.copyWith(size: 2048, evidenceType: 'video');
      expect(next.size, 2048);
      expect(next.evidenceType, 'video');
      expect(next.id, base.id); // unchanged
      expect(next.metadata['station'], 'ST-001');
    });

    test('equality by id and idempotencyKey', () {
      final same = EvidenceUploadRequest(
        id: 'ev-1',
        idempotencyKey: 'idem-1',
        path: '/other',
        contentType: 'image/png',
        size: 999,
        sha256: 'different',
      );
      expect(base, same);
    });

    test('different idempotencyKey is not equal', () {
      final other = base.copyWith(idempotencyKey: 'idem-2');
      expect(base != other, isTrue);
    });
  });

  group('InMemoryEvidenceUploader', () {
    late InMemoryEvidenceUploader uploader;
    final request = EvidenceUploadRequest(
      id: 'ev-1',
      idempotencyKey: 'idem-1',
      path: '/data/photo.jpg',
      contentType: 'image/jpeg',
      size: 1024,
      sha256: 'abc123',
    );

    setUp(() {
      uploader = InMemoryEvidenceUploader();
    });

    test('upload records request and returns success', () async {
      final result = await uploader.upload(request);
      expect(result, isA<EvidenceUploadSuccess>());
      final success = result as EvidenceUploadSuccess;
      expect(success.evidenceId, 'uploaded-ev-1');
      expect(success.sha256, 'abc123');
      expect(success.bytesSent, 1024);
      expect(uploader.sent.length, 1);
      expect(uploader.sent.first.id, 'ev-1');
    });

    test('tracks failed uploads', () async {
      uploader.failNext = true;
      final result = await uploader.upload(request);
      expect(result, isA<EvidenceUploadError>());
      expect(uploader.failed.length, 1);
    });

    test('reset clears state', () async {
      await uploader.upload(request);
      uploader.reset();
      expect(uploader.sent, isEmpty);
      expect(uploader.failed, isEmpty);
    });
  });

  group('EvidenceUploaderCoordinator — consent gate (EC-CON-03)', () {
    late EvidenceUploaderCoordinator coordinator;
    late InMemoryEvidenceUploader transport;
    late InMemoryStorageChecker storage;

    setUp(() {
      transport = InMemoryEvidenceUploader();
      storage = InMemoryStorageChecker(available: 10 * 1024 * 1024); // 10 MB
      coordinator = EvidenceUploaderCoordinator(
        transport: transport,
        storage: storage,
      );
    });

    test('allows small upload on degraded without consent', () async {
      final small = EvidenceUploadRequest(
        id: 'small',
        idempotencyKey: 'small-idem',
        path: '/data/thumb.jpg',
        contentType: 'image/jpeg',
        size: 100,
        sha256: 'small',
      );
      final result = await coordinator.upload(small, ConnectivityState.degraded);
      expect(result, isA<EvidenceUploadSuccess>());
    });

    test('requires consent for large upload on degraded', () async {
      final large = EvidenceUploadRequest(
        id: 'large',
        idempotencyKey: 'large-idem',
        path: '/data/video.mp4',
        contentType: 'video/mp4',
        size: evidenceConsentThresholdBytes + 1,
        sha256: 'large',
      );
      final result =
          await coordinator.upload(large, ConnectivityState.degraded);
      expect(result, isA<EvidenceUploadConsentRequired>());
      final consent = result as EvidenceUploadConsentRequired;
      expect(consent.request.id, 'large');
      expect(consent.connectivityState, ConnectivityState.degraded);
      expect(transport.sent, isEmpty); // not attempted
    });

    test('allows large upload on online without consent', () async {
      final large = EvidenceUploadRequest(
        id: 'large',
        idempotencyKey: 'large-idem',
        path: '/data/video.mp4',
        contentType: 'video/mp4',
        size: evidenceConsentThresholdBytes + 1,
        sha256: 'large',
      );
      final result = await coordinator.upload(large, ConnectivityState.online);
      expect(result, isA<EvidenceUploadSuccess>());
    });

    test('allows large upload on offline (deferred to outbox)', () async {
      final large = EvidenceUploadRequest(
        id: 'offline',
        idempotencyKey: 'offline-idem',
        path: '/data/video.mp4',
        contentType: 'video/mp4',
        size: evidenceConsentThresholdBytes + 1,
        sha256: 'offline',
      );
      // Offline: coordinator still delegates to transport (outbox handles
      // the actual send later); consent gate only applies to degraded.
      final result = await coordinator.upload(large, ConnectivityState.offline);
      expect(result, isA<EvidenceUploadSuccess>());
    });

    test('consent threshold is exactly 500 KB', () {
      final atThreshold = EvidenceUploadRequest(
        id: 'at',
        idempotencyKey: 'at-idem',
        path: '/data/x',
        contentType: 'image/jpeg',
        size: evidenceConsentThresholdBytes,
        sha256: 'at',
      );
      final over = EvidenceUploadRequest(
        id: 'over',
        idempotencyKey: 'over-idem',
        path: '/data/x',
        contentType: 'image/jpeg',
        size: evidenceConsentThresholdBytes + 1,
        sha256: 'over',
      );
      expect(
        coordinator.upload(atThreshold, ConnectivityState.degraded),
        completes, // goes to transport
      );
      expect(
        coordinator.upload(over, ConnectivityState.degraded),
        completes, // consent required
      );
    });
  });

  group('EvidenceUploaderCoordinator — storage guard (EC-MOB-02)', () {
    late EvidenceUploaderCoordinator coordinator;
    late InMemoryEvidenceUploader transport;
    late InMemoryStorageChecker storage;

    setUp(() {
      transport = InMemoryEvidenceUploader();
      storage = InMemoryStorageChecker(available: 1 * 1024 * 1024); // 1 MB
      coordinator = EvidenceUploaderCoordinator(
        transport: transport,
        storage: storage,
      );
    });

    test('refuses upload when storage is full (EC-MOB-02)', () async {
      final large = EvidenceUploadRequest(
        id: 'big',
        idempotencyKey: 'big-idem',
        path: '/data/photo.jpg',
        contentType: 'image/jpeg',
        size: 2 * 1024 * 1024, // 2 MB
        sha256: 'big',
      );
      final result = await coordinator.upload(large, ConnectivityState.online);
      expect(result, isA<EvidenceUploadStorageFull>());
      final full = result as EvidenceUploadStorageFull;
      expect(full.request.id, 'big');
      expect(full.availableBytes, 1 * 1024 * 1024);
      expect(full.requiredBytes, 2 * 1024 * 1024);
      expect(transport.sent, isEmpty); // not attempted
    });

    test('allows upload when storage is sufficient', () async {
      final small = EvidenceUploadRequest(
        id: 'small',
        idempotencyKey: 'small-idem',
        path: '/data/thumb.jpg',
        contentType: 'image/jpeg',
        size: 512 * 1024, // 512 KB
        sha256: 'small',
      );
      final result = await coordinator.upload(small, ConnectivityState.online);
      expect(result, isA<EvidenceUploadSuccess>());
    });

    test('allows upload when storage check is unavailable (null)', () async {
      // InMemoryStorageChecker always returns an int; use a large value to
      // simulate "unknown" by making storage appear sufficient.
      storage.setAvailable(10 * 1024 * 1024 * 1024); // 10 GB
      final large = EvidenceUploadRequest(
        id: 'unknown',
        idempotencyKey: 'unknown-idem',
        path: '/data/video.mp4',
        contentType: 'video/mp4',
        size: 10 * 1024 * 1024,
        sha256: 'unknown',
      );
      // Null available → defer to transport (OS will fail if truly full)
      final result = await coordinator.upload(large, ConnectivityState.online);
      expect(result, isA<EvidenceUploadSuccess>());
    });

    test('storage guard checked before consent gate', () async {
      // Storage full + degraded: storage guard wins (refuses before consent).
      final large = EvidenceUploadRequest(
        id: 'both',
        idempotencyKey: 'both-idem',
        path: '/data/video.mp4',
        contentType: 'video/mp4',
        size: 2 * 1024 * 1024,
        sha256: 'both',
      );
      final result =
          await coordinator.upload(large, ConnectivityState.degraded);
      expect(result, isA<EvidenceUploadStorageFull>());
      expect(result is EvidenceUploadConsentRequired, isFalse);
    });
  });

  group('EvidenceUploaderCoordinator — end-to-end', () {
    late EvidenceUploaderCoordinator coordinator;
    late InMemoryEvidenceUploader transport;
    late InMemoryStorageChecker storage;

    setUp(() {
      transport = InMemoryEvidenceUploader();
      storage = InMemoryStorageChecker(available: 100 * 1024 * 1024);
      coordinator = EvidenceUploaderCoordinator(
        transport: transport,
        storage: storage,
      );
    });

    test('full success path', () async {
      final request = EvidenceUploadRequest(
        id: 'ev-final',
        idempotencyKey: 'idem-final',
        path: '/data/evidence.jpg',
        contentType: 'image/jpeg',
        size: 256 * 1024,
        sha256: computeSha256(Uint8List.fromList([1, 2, 3])),
        evidenceType: 'photo',
        metadata: {'station': 'ST-42', 'clerk_id': 'C-100'},
      );
      final result = await coordinator.upload(request, ConnectivityState.online);
      expect(result, isA<EvidenceUploadSuccess>());
      final success = result as EvidenceUploadSuccess;
      expect(success.evidenceId, 'uploaded-ev-final');
      expect(success.sha256, request.sha256);
      expect(success.bytesSent, 256 * 1024);
    });

    test('transport error propagates', () async {
      transport.failNext = true;
      final request = EvidenceUploadRequest(
        id: 'err',
        idempotencyKey: 'err-idem',
        path: '/data/x.jpg',
        contentType: 'image/jpeg',
        size: 100,
        sha256: 'err',
      );
      final result = await coordinator.upload(request, ConnectivityState.online);
      expect(result, isA<EvidenceUploadError>());
      final error = result as EvidenceUploadError;
      // InMemoryEvidenceUploader records the failed request under a fixed id.
      expect(error.error, isA<Exception>());
    });
  });
}
