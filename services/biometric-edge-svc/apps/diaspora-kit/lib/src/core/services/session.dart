/// Diaspora Kit session (M24). Roles: BVR_KIOSK / DIASPORA_READ
/// Per plan.md M3: Fabricated HSM → real key + deferred batch upload
library;

import 'dart:convert';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:uuid/uuid.dart';

class DiasporaSessionState {
  final bool authenticated;
  final String? userId;
  final String? role;
  final String? kioskSerial;
  final bool deferredBatchUpload;
  final DateTime expiresAt;
  final String sessionId;

  const DiasporaSessionState({
    this.authenticated = false,
    this.userId,
    this.role,
    this.kioskSerial,
    this.deferredBatchUpload = false,
    required this.expiresAt,
    required this.sessionId,
  });

  factory DiasporaSessionState.initial() => DiasporaSessionState(
    expiresAt: DateTime.now(),
    sessionId: const Uuid().v7(),
  );

  factory DiasporaSessionState.authenticated({required String userId, String? kioskSerial, bool deferredBatch = false}) =>
    DiasporaSessionState(
      authenticated: true,
      userId: userId,
      role: 'diaspora_kiosk',
      kioskSerial: kioskSerial,
      deferredBatchUpload: deferredBatch,
      expiresAt: DateTime.now().add(const Duration(hours: 8)),
      sessionId: const Uuid().v7(),
    );

  DiasporaSessionState copyWith({bool? authenticated, String? userId, String? role, String? kioskSerial, bool? deferredBatchUpload, DateTime? expiresAt, String? sessionId}) =>
    DiasporaSessionState(
      authenticated: authenticated ?? this.authenticated,
      userId: userId ?? this.userId,
      role: role ?? this.role,
      kioskSerial: kioskSerial ?? this.kioskSerial,
      deferredBatchUpload: deferredBatchUpload ?? this.deferredBatchUpload,
      expiresAt: expiresAt ?? this.expiresAt,
      sessionId: sessionId ?? this.sessionId,
    );

  bool get isExpired => DateTime.now().isAfter(expiresAt);
}

class DiasporaSessionService {
  final FlutterSecureStorage _store;
  DiasporaSessionService({FlutterSecureStorage? s}) : _store = s ?? const FlutterSecureStorage();

  Future<DiasporaSessionState> load() async {
    try {
      final d = await _store.read(key: 'diaspora_session');
      if (d == null) return DiasporaSessionState.initial();
      final data = jsonDecode(d) as Map<String, dynamic>;
      final s = DiasporaSessionState(
        authenticated: data['authenticated'] == true,
        userId: data['user_id'] as String?,
        role: data['role'] as String?,
        kioskSerial: data['kiosk_serial'] as String?,
        deferredBatchUpload: data['deferred_batch_upload'] == true,
        expiresAt: DateTime.parse(data['expires_at']),
        sessionId: data['session_id'] as String? ?? const Uuid().v7(),
      );
      if (s.isExpired) return DiasporaSessionState.initial();
      return s;
    } catch (_) { return DiasporaSessionState.initial(); }
  }

  Future<void> save(DiasporaSessionState s) async => _store.write(key: 'diaspora_session', value: jsonEncode({
    'authenticated': s.authenticated,
    'user_id': s.userId,
    'role': s.role,
    'kiosk_serial': s.kioskSerial,
    'deferred_batch_upload': s.deferredBatchUpload,
    'expires_at': s.expiresAt.toIso8601String(),
    'session_id': s.sessionId,
  }));

  Future<void> clear() async => _store.delete(key: 'diaspora_session');
}
