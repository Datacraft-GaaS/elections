import 'package:crypto/crypto.dart' as crypto show sha256;
import 'package:flutter/foundation.dart';

import '../connectivity/connectivity_probe.dart';

/// Result of an evidence upload attempt (plan §2.5, EC-CON-03, EC-MOB-02).
sealed class EvidenceUploadResult {
  const EvidenceUploadResult();
}

/// Upload succeeded; server returned the evidence id.
class EvidenceUploadSuccess extends EvidenceUploadResult {
  const EvidenceUploadSuccess({
    required this.evidenceId,
    required this.sha256,
    required this.bytesSent,
  });

  final String evidenceId;
  final String sha256;
  final int bytesSent;
}

/// Upload deferred: degraded connectivity and the file exceeds the consent
/// threshold (EC-CON-03). Caller must surface a consent prompt before retry.
class EvidenceUploadConsentRequired extends EvidenceUploadResult {
  const EvidenceUploadConsentRequired({
    required this.request,
    required this.connectivityState,
  });

  final EvidenceUploadRequest request;
  final ConnectivityState connectivityState;
}

/// Upload refused: device storage is full (EC-MOB-02). Text-only outbox
/// entries may still be queued; evidence uploads are blocked.
class EvidenceUploadStorageFull extends EvidenceUploadResult {
  const EvidenceUploadStorageFull({
    required this.request,
    required this.availableBytes,
    required this.requiredBytes,
  });

  final EvidenceUploadRequest request;
  final int availableBytes;
  final int requiredBytes;
}

/// Upload failed with a non-retryable error (bad request, auth, etc.).
class EvidenceUploadError extends EvidenceUploadResult {
  const EvidenceUploadError({
    required this.request,
    required this.error,
    this.detail,
  });

  final EvidenceUploadRequest request;
  final Object error;
  final String? detail;
}

/// Evidence upload request (plan §2.5, module FRs for photo/video evidence).
@immutable
class EvidenceUploadRequest {
  const EvidenceUploadRequest({
    required this.id,
    required this.idempotencyKey,
    required this.path,
    required this.contentType,
    required this.size,
    required this.sha256,
    this.evidenceType = 'photo',
    this.metadata = const {},
  });

  final String id;
  final String idempotencyKey;

  /// Filesystem path to the evidence asset.
  final String path;

  /// MIME type, e.g. `image/jpeg`, `video/mp4`.
  final String contentType;

  /// Size in bytes (must match sha256 computation on the actual bytes).
  final int size;

  /// SHA-256 hex digest of the raw bytes — integrity footer (EC-MOB-01).
  final String sha256;

  /// Evidence category: photo, video, audio, document.
  final String evidenceType;

  /// Arbitrary metadata attached to the upload (station code, clerk id, etc.).
  final Map<String, Object?> metadata;

  EvidenceUploadRequest copyWith({
    String? id,
    String? idempotencyKey,
    String? path,
    String? contentType,
    int? size,
    String? sha256,
    String? evidenceType,
    Map<String, Object?>? metadata,
  }) =>
      EvidenceUploadRequest(
        id: id ?? this.id,
        idempotencyKey: idempotencyKey ?? this.idempotencyKey,
        path: path ?? this.path,
        contentType: contentType ?? this.contentType,
        size: size ?? this.size,
        sha256: sha256 ?? this.sha256,
        evidenceType: evidenceType ?? this.evidenceType,
        metadata: metadata ?? this.metadata,
      );

  @override
  bool operator ==(Object other) =>
      other is EvidenceUploadRequest &&
      other.id == id &&
      other.idempotencyKey == idempotencyKey;

  @override
  int get hashCode => Object.hash(id, idempotencyKey);
}

/// Threshold above which a consent prompt is required on degraded connectivity
/// (EC-CON-03). Files below this size may upload without explicit consent.
const evidenceConsentThresholdBytes = 500 * 1024; // 500 KB

/// Abstract evidence upload transport. Apps bind a real implementation that
/// posts multipart to the BFF; tests bind [InMemoryEvidenceUploader].
abstract class EvidenceUploader {
  /// Attempt to upload [request]. Returns a sealed result describing the
  /// outcome: success, consent required, storage full, or error.
  Future<EvidenceUploadResult> upload(EvidenceUploadRequest request);
}

/// In-memory test double. Simulates a successful upload and tracks sent
/// requests for test assertions.
class InMemoryEvidenceUploader implements EvidenceUploader {
  final List<EvidenceUploadRequest> sent = [];
  final List<EvidenceUploadRequest> failed = [];
  bool failNext = false;
  String? nextEvidenceId;

  @override
  Future<EvidenceUploadResult> upload(EvidenceUploadRequest request) async {
    sent.add(request);
    if (failNext) {
      failed.add(request);
      return EvidenceUploadError(
        request: EvidenceUploadRequest(
          id: 'fail',
          idempotencyKey: 'fail-idem',
          path: '/dev/null',
          contentType: 'application/octet-stream',
          size: 0,
          sha256: '',
        ),
        error: Exception('simulated upload failure'),
      );
    }
    final evidenceId = nextEvidenceId ?? 'uploaded-${request.id}';
    return EvidenceUploadSuccess(
      evidenceId: evidenceId,
      sha256: request.sha256,
      bytesSent: request.size,
    );
  }

  void reset() {
    sent.clear();
    failed.clear();
    failNext = false;
    nextEvidenceId = null;
  }
}

/// Computes the SHA-256 hex digest of a byte payload (plan §2.3, EC-MOB-01).
String computeSha256(Uint8List bytes) =>
    crypto.sha256.convert(bytes).toString();

/// Storage checker abstraction. At the app layer this reads available space
/// from the OS; tests bind a fake.
abstract class StorageChecker {
  /// Returns available bytes, or null if the check is unavailable.
  Future<int?> availableBytes();
}

/// In-memory storage checker for tests.
class InMemoryStorageChecker implements StorageChecker {
  int? _available;

  InMemoryStorageChecker({int? available}) : _available = available;

  void setAvailable(int bytes) => _available = bytes;

  @override
  Future<int?> availableBytes() async => _available;
}

/// Wall-clock storage checker (production).
///
/// [availableBytesDelegate] must be bound by the app, because Flutter has no
/// built-in free-space API — it is a platform channel or a plugin. It is a
/// required argument rather than defaulting to null because null means
/// "unavailable", and every caller skips the EC-MOB-02 storage guard when the
/// check is unavailable. An earlier version returned null unconditionally, so
/// the guard was live in tests and dead in production.
class RealStorageChecker implements StorageChecker {
  RealStorageChecker({required this.availableBytesDelegate});

  /// Reads free space on the volume holding the app's data.
  ///
  /// Return null only when the figure genuinely cannot be read; a caller that
  /// cannot report storage should let uploads proceed rather than block an
  /// election, and the audit trail records the upload either way.
  final Future<int?> Function() availableBytesDelegate;

  @override
  Future<int?> availableBytes() => availableBytesDelegate();
}

/// Upload coordinator: applies consent gate (EC-CON-03) and storage guard
/// (EC-MOB-02) before delegating to the transport.
///
/// Production usage:
/// ```dart
/// final uploader = EvidenceUploaderCoordinator(
///   transport: realBffUploader,
///   storage: RealStorageChecker(
///     availableBytesDelegate: platformChannelFreeBytes,
///   ),
/// );
/// final result = await uploader.upload(request, connectivityState);
/// ```
class EvidenceUploaderCoordinator {
  EvidenceUploaderCoordinator({
    required this.transport,
    required this.storage,
    this.degradedThreshold = evidenceConsentThresholdBytes,
  });

  final EvidenceUploader transport;
  final StorageChecker storage;
  final int degradedThreshold;

  /// Attempt upload with consent and storage gates applied.
  ///
  /// - If connectivity is degraded and the file exceeds [degradedThreshold],
  ///   returns [EvidenceUploadConsentRequired] — the caller must surface a
  ///   consent prompt before retrying.
  /// - If available storage is less than the request size, returns
  ///   [EvidenceUploadStorageFull] — evidence uploads are refused; text-only
  ///   outbox entries may still be queued (EC-MOB-02).
  /// - Otherwise delegates to [transport.upload].
  Future<EvidenceUploadResult> upload(
    EvidenceUploadRequest request,
    ConnectivityState connectivityState,
  ) async {
    // EC-MOB-02: storage-full refusal — checked before consent so a
    // storage-full device never surfaces a consent prompt it cannot act on.
    final available = await storage.availableBytes();
    if (available != null && available < request.size) {
      return EvidenceUploadStorageFull(
        request: request,
        availableBytes: available,
        requiredBytes: request.size,
      );
    }

    // EC-CON-03: consent gate on degraded connectivity for large uploads.
    if (connectivityState == ConnectivityState.degraded &&
        request.size > degradedThreshold) {
      return EvidenceUploadConsentRequired(
        request: request,
        connectivityState: connectivityState,
      );
    }
    return transport.upload(request);
  }
}
