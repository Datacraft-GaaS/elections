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

/// BVR Kit session (M01). Roles: BVR_OPERATOR
/// Per plan.md M3: MRZ/IPRS biometric capture, document scanning, voter verification
library;

import 'dart:convert';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:uuid/uuid.dart';

class BvrSessionState {
  final bool authenticated;
  final String? accessToken;
  final String? userId;
  final String? role;
  final String? stationCode;
  final String? deviceId;
  final DateTime expiresAt;
  final String sessionId;

  const BvrSessionState({
    this.authenticated = false,
    this.accessToken,
    this.userId,
    this.role,
    this.stationCode,
    this.deviceId,
    required this.expiresAt,
    required this.sessionId,
  });

  factory BvrSessionState.initial() => BvrSessionState(
    expiresAt: DateTime.now(),
    sessionId: const Uuid().v7(),
  );

  factory BvrSessionState.authenticated({required String userId, String? stationCode, String? deviceId}) =>
    BvrSessionState(
      authenticated: true,
      userId: userId,
      role: 'bvr_operator',
      stationCode: stationCode,
      deviceId: deviceId,
      expiresAt: DateTime.now().add(const Duration(hours: 12)),
      sessionId: const Uuid().v7(),
    );

  BvrSessionState copyWith({
    bool? authenticated, String? accessToken, String? userId,
    String? role, String? stationCode, String? deviceId,
    DateTime? expiresAt, String? sessionId,
  }) => BvrSessionState(
    authenticated: authenticated ?? this.authenticated,
    accessToken: accessToken ?? this.accessToken,
    userId: userId ?? this.userId,
    role: role ?? this.role,
    stationCode: stationCode ?? this.stationCode,
    deviceId: deviceId ?? this.deviceId,
    expiresAt: expiresAt ?? this.expiresAt,
    sessionId: sessionId ?? this.sessionId,
  );

  bool get isExpired => DateTime.now().isAfter(expiresAt);
}

class BvrSessionService {
  final FlutterSecureStorage _store;
  BvrSessionService({FlutterSecureStorage? s}) : _store = s ?? const FlutterSecureStorage();

  Future<BvrSessionState> load() async {
    try {
      final d = await _store.read(key: 'bvr_session');
      if (d == null) return BvrSessionState.initial();
      final data = jsonDecode(d) as Map<String, dynamic>;
      final s = BvrSessionState(
        authenticated: data['authenticated'] == true,
        accessToken: data['access_token'] as String?,
        userId: data['user_id'] as String?,
        role: data['role'] as String?,
        stationCode: data['station_code'] as String?,
        deviceId: data['device_id'] as String?,
        expiresAt: DateTime.parse(data['expires_at']),
        sessionId: data['session_id'] as String? ?? const Uuid().v7(),
      );
      if (s.isExpired) return BvrSessionState.initial();
      return s;
    } catch (_) { return BvrSessionState.initial(); }
  }

  Future<void> save(BvrSessionState s) async => _store.write(key: 'bvr_session', value: jsonEncode({
    'authenticated': s.authenticated,
    'access_token': s.accessToken,
    'user_id': s.userId,
    'role': s.role,
    'station_code': s.stationCode,
    'device_id': s.deviceId,
    'expires_at': s.expiresAt.toIso8601String(),
    'session_id': s.sessionId,
  }));

  Future<void> clear() async => _store.delete(key: 'bvr_session');
}
