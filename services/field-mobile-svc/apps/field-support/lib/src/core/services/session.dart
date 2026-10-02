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

/// Session for field-support (M32). Roles from module spec.
library;

import 'dart:convert';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:local_auth/local_auth.dart';
import 'package:uuid/uuid.dart';

enum SessionPhase { unauthenticated, biometricStepUp, pinStepUp, authenticated, sessionActive, locked, expired }

class SessionState {
  final SessionPhase phase;
  final String? accessToken, refreshToken, userId, role, engineerId;
  final int pinFailures;
  final DateTime? lockedUntil;
  final DateTime expiresAt;
  final String sessionId;

  const SessionState({
    this.phase = SessionPhase.unauthenticated,
    this.accessToken, this.refreshToken, this.userId, this.role,
    this.engineerId, this.pinFailures = 0,
    this.lockedUntil, required this.expiresAt, required this.sessionId,
  });

  factory SessionState.initial() => SessionState(sessionId: const Uuid().v7(), expiresAt: DateTime.now());

  factory SessionState.authenticated({required String userId, String? role, required String accessToken, required String refreshToken, String? engineerId}) =>
    SessionState(
      phase: SessionPhase.authenticated,
      accessToken: accessToken,
      refreshToken: refreshToken,
      userId: userId,
      role: role ?? 'datacraft_engineer',
      engineerId: engineerId,
      sessionId: const Uuid().v7(),
      expiresAt: DateTime.now().add(const Duration(hours: 5)),
    );

  SessionState afterPinFailure() {
    final nf = pinFailures + 1;
    return SessionState(
      phase: nf >= 5 ? SessionPhase.locked : phase,
      accessToken: accessToken, refreshToken: refreshToken, userId: userId,
      role: role, engineerId: engineerId, pinFailures: nf,
      lockedUntil: nf >= 5 ? DateTime.now().add(const Duration(minutes: 30)) : null,
      expiresAt: expiresAt, sessionId: sessionId,
    );
  }

  bool get isLocked => lockedUntil != null && DateTime.now().isBefore(lockedUntil!);

  SessionState copyWith({SessionPhase? phase, String? accessToken, String? refreshToken, String? userId, String? role, String? engineerId, int? pinFailures, DateTime? lockedUntil, DateTime? expiresAt, String? sessionId}) =>
    SessionState(
      phase: phase ?? this.phase,
      accessToken: accessToken ?? this.accessToken,
      refreshToken: refreshToken ?? this.refreshToken,
      userId: userId ?? this.userId,
      role: role ?? this.role,
      engineerId: engineerId ?? this.engineerId,
      pinFailures: pinFailures ?? this.pinFailures,
      lockedUntil: lockedUntil ?? this.lockedUntil,
      expiresAt: expiresAt ?? this.expiresAt,
      sessionId: sessionId ?? this.sessionId,
    );
}

class SessionService {
  final FlutterSecureStorage _store;
  final LocalAuthentication _auth;
  SessionService({FlutterSecureStorage? s, LocalAuthentication? a}) : _store = s ?? const FlutterSecureStorage(), _auth = a ?? LocalAuthentication();

  Future<SessionState> load() async {
    try {
      final d = await _store.read(key: 'field_support_session');
      if (d == null) return SessionState.initial();
      final data = jsonDecode(d) as Map<String, dynamic>;
      return SessionState(
        phase: SessionPhase.sessionActive,
        accessToken: data['access_token'] as String?,
        refreshToken: data['refresh_token'] as String?,
        userId: data['user_id'] as String?,
        role: data['role'] as String?,
        engineerId: data['engineer_id'] as String?,
        pinFailures: data['pin_failures'] as int? ?? 0,
        lockedUntil: data['locked_until'] != null ? DateTime.parse(data['locked_until']) : null,
        expiresAt: DateTime.parse(data['expires_at']),
        sessionId: data['session_id'] as String? ?? const Uuid().v7(),
      );
    } catch (_) { return SessionState.initial(); }
  }

  Future<void> save(SessionState s) async => _store.write(key: 'field_support_session', value: jsonEncode({
    'access_token': s.accessToken, 'refresh_token': s.refreshToken,
    'user_id': s.userId, 'role': s.role, 'engineer_id': s.engineerId,
    'pin_failures': s.pinFailures,
    'locked_until': s.lockedUntil?.toIso8601String(),
    'expires_at': s.expiresAt.toIso8601String(),
    'session_id': s.sessionId,
  }));

  Future<void> clear() async => await _store.delete(key: 'field_support_session');
  Future<bool> canCheckBiometrics() async => await _auth.canCheckBiometrics || await _auth.isDeviceSupported();
  Future<bool> authBio() async => await _auth.authenticate(localizedReason: 'Field engineer identity', options: const AuthenticationOptions(biometricOnly: true));
}
