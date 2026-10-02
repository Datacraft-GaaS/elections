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

/// BVI Kit session (M05). Roles: BVI_OPERATOR
/// Per plan.md M3: NTS (Nintex Trusted Service) + GNSS + fingerprint capture
library;

import 'dart:convert';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:uuid/uuid.dart';

class BviSessionState {
  final bool authenticated;
  final String? userId;
  final String? role;
  final String? stationCode;
  final String? kioskSerial;
  final DateTime expiresAt;
  final String sessionId;

  const BviSessionState({
    this.authenticated = false,
    this.userId,
    this.role,
    this.stationCode,
    this.kioskSerial,
    required this.expiresAt,
    required this.sessionId,
  });

  factory BviSessionState.initial() => BviSessionState(
    expiresAt: DateTime.now(),
    sessionId: const Uuid().v7(),
  );

  factory BviSessionState.authenticated({required String userId, String? stationCode, String? kioskSerial}) =>
    BviSessionState(
      authenticated: true,
      userId: userId,
      role: 'bvi_operator',
      stationCode: stationCode,
      kioskSerial: kioskSerial,
      expiresAt: DateTime.now().add(const Duration(hours: 12)),
      sessionId: const Uuid().v7(),
    );

  BviSessionState copyWith({bool? authenticated, String? userId, String? role, String? stationCode, String? kioskSerial, DateTime? expiresAt, String? sessionId}) =>
    BviSessionState(
      authenticated: authenticated ?? this.authenticated,
      userId: userId ?? this.userId,
      role: role ?? this.role,
      stationCode: stationCode ?? this.stationCode,
      kioskSerial: kioskSerial ?? this.kioskSerial,
      expiresAt: expiresAt ?? this.expiresAt,
      sessionId: sessionId ?? this.sessionId,
    );

  bool get isExpired => DateTime.now().isAfter(expiresAt);
}

class BviSessionService {
  final FlutterSecureStorage _store;
  BviSessionService({FlutterSecureStorage? s}) : _store = s ?? const FlutterSecureStorage();

  Future<BviSessionState> load() async {
    try {
      final d = await _store.read(key: 'bvi_session');
      if (d == null) return BviSessionState.initial();
      final data = jsonDecode(d) as Map<String, dynamic>;
      return BviSessionState(
        authenticated: data['authenticated'] == true,
        userId: data['user_id'] as String?,
        role: data['role'] as String?,
        stationCode: data['station_code'] as String?,
        kioskSerial: data['kiosk_serial'] as String?,
        expiresAt: DateTime.parse(data['expires_at']),
        sessionId: data['session_id'] as String? ?? const Uuid().v7(),
      );
    } catch (_) { return BviSessionState.initial(); }
  }

  Future<void> save(BviSessionState s) async => _store.write(key: 'bvi_session', value: jsonEncode({
    'authenticated': s.authenticated,
    'user_id': s.userId,
    'role': s.role,
    'station_code': s.stationCode,
    'kiosk_serial': s.kioskSerial,
    'expires_at': s.expiresAt.toIso8601String(),
    'session_id': s.sessionId,
  }));

  Future<void> clear() async => _store.delete(key: 'bvi_session');
}
