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

/// RTS Kit session (M08). Roles: RTS_OPERATOR
/// Per plan.md M3: SMS fallback stub → real status responses
library;

import 'dart:convert';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:uuid/uuid.dart';

class RtsSessionState {
  final bool authenticated;
  final String? userId;
  final String? role;
  final String? stationCode;
  final String? smsFallbackActive;
  final DateTime expiresAt;
  final String sessionId;

  const RtsSessionState({
    this.authenticated = false,
    this.userId,
    this.role,
    this.stationCode,
    this.smsFallbackActive,
    required this.expiresAt,
    required this.sessionId,
  });

  factory RtsSessionState.initial() => RtsSessionState(
    expiresAt: DateTime.now(),
    sessionId: const Uuid().v7(),
  );

  factory RtsSessionState.authenticated({required String userId, String? stationCode, bool smsFallback = false}) =>
    RtsSessionState(
      authenticated: true,
      userId: userId,
      role: 'rts_operator',
      stationCode: stationCode,
      smsFallbackActive: smsFallback ? 'active' : null,
      expiresAt: DateTime.now().add(const Duration(hours: 8)),
      sessionId: const Uuid().v7(),
    );

  RtsSessionState copyWith({bool? authenticated, String? userId, String? role, String? stationCode, String? smsFallbackActive, DateTime? expiresAt, String? sessionId}) =>
    RtsSessionState(
      authenticated: authenticated ?? this.authenticated,
      userId: userId ?? this.userId,
      role: role ?? this.role,
      stationCode: stationCode ?? this.stationCode,
      smsFallbackActive: smsFallbackActive ?? this.smsFallbackActive,
      expiresAt: expiresAt ?? this.expiresAt,
      sessionId: sessionId ?? this.sessionId,
    );

  bool get isExpired => DateTime.now().isAfter(expiresAt);
}

class RtsSessionService {
  final FlutterSecureStorage _store;
  RtsSessionService({FlutterSecureStorage? s}) : _store = s ?? const FlutterSecureStorage();

  Future<RtsSessionState> load() async {
    try {
      final d = await _store.read(key: 'rts_session');
      if (d == null) return RtsSessionState.initial();
      final data = jsonDecode(d) as Map<String, dynamic>;
      final s = RtsSessionState(
        authenticated: data['authenticated'] == true,
        userId: data['user_id'] as String?,
        role: data['role'] as String?,
        stationCode: data['station_code'] as String?,
        smsFallbackActive: data['sms_fallback_active'] as String?,
        expiresAt: DateTime.parse(data['expires_at']),
        sessionId: data['session_id'] as String? ?? const Uuid().v7(),
      );
      if (s.isExpired) return RtsSessionState.initial();
      return s;
    } catch (_) { return RtsSessionState.initial(); }
  }

  Future<void> save(RtsSessionState s) async => _store.write(key: 'rts_session', value: jsonEncode({
    'authenticated': s.authenticated,
    'user_id': s.userId,
    'role': s.role,
    'station_code': s.stationCode,
    'sms_fallback_active': s.smsFallbackActive,
    'expires_at': s.expiresAt.toIso8601String(),
    'session_id': s.sessionId,
  }));

  Future<void> clear() async => _store.delete(key: 'rts_session');
}
