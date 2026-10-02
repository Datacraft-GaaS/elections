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

/// Anonymous session for citizen-verify (M13/M29).
/// No MDM, soft attestation only — but still bounded by rate limits.
/// Per plan.md §781: citizen-verify uses `attestation: soft-warn` tier;
/// never blocks feature access. Session is anonymous with rate-limit token.

import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:uuid/uuid.dart';

class AnonSessionState {
  final String sessionId;
  final DateTime createdAt;
  final String? rateLimitToken;
  final bool softAttestationWarned;
  final String? phoneNumber;
  final bool otpVerified;
  final int otpAttemptsToday;
  final int registerLookupsToday;
  final int objectionsToday;

  const AnonSessionState({
    required this.sessionId,
    required this.createdAt,
    this.rateLimitToken,
    this.softAttestationWarned = false,
    this.phoneNumber,
    this.otpVerified = false,
    this.otpAttemptsToday = 0,
    this.registerLookupsToday = 0,
    this.objectionsToday = 0,
  });

  factory AnonSessionState.initial() => AnonSessionState(
    sessionId: const Uuid().v7(),
    createdAt: DateTime.now(),
  );

  AnonSessionState copyWith({
    String? sessionId,
    DateTime? createdAt,
    String? rateLimitToken,
    bool? softAttestationWarned,
    String? phoneNumber,
    bool? otpVerified,
    int? otpAttemptsToday,
    int? registerLookupsToday,
    int? objectionsToday,
  }) => AnonSessionState(
    sessionId: sessionId ?? this.sessionId,
    createdAt: createdAt ?? this.createdAt,
    rateLimitToken: rateLimitToken ?? this.rateLimitToken,
    softAttestationWarned: softAttestationWarned ?? this.softAttestationWarned,
    phoneNumber: phoneNumber ?? this.phoneNumber,
    otpVerified: otpVerified ?? this.otpVerified,
    otpAttemptsToday: otpAttemptsToday ?? this.otpAttemptsToday,
    registerLookupsToday: registerLookupsToday ?? this.registerLookupsToday,
    objectionsToday: objectionsToday ?? this.objectionsToday,
  );

  bool get isOtpValid => otpVerified;
  bool get isRateLimited => registerLookupsToday >= 20; // M13: 20/day/session
  bool get isObjectionRateLimited => objectionsToday >= 5; // M13: 5/day/session
}

class AnonSessionService {
  final FlutterSecureStorage _store;
  AnonSessionService({FlutterSecureStorage? s}) : _store = s ?? const FlutterSecureStorage();

  Future<AnonSessionState> load() async {
    try {
      final d = await _store.read(key: 'citizen_anon_session');
      if (d == null) return AnonSessionState.initial();
      final j = _decode(d);
      return AnonSessionState(
        sessionId: j['session_id'] as String? ?? const Uuid().v7(),
        createdAt: DateTime.parse(j['created_at'] as String),
        rateLimitToken: j['rate_limit_token'] as String?,
        softAttestationWarned: j['soft_attestation_warned'] == true,
        phoneNumber: j['phone_number'] as String?,
        otpVerified: j['otp_verified'] == true,
        otpAttemptsToday: (j['otp_attempts_today'] as num?)?.toInt() ?? 0,
        registerLookupsToday: (j['register_lookups_today'] as num?)?.toInt() ?? 0,
        objectionsToday: (j['objections_today'] as num?)?.toInt() ?? 0,
      );
    } catch (_) { return AnonSessionState.initial(); }
  }

  Future<void> save(AnonSessionState s) async => _store.write(
    key: 'citizen_anon_session',
    value: _encode({
      'session_id': s.sessionId,
      'created_at': s.createdAt.toIso8601String(),
      'rate_limit_token': s.rateLimitToken,
      'soft_attestation_warned': s.softAttestationWarned,
      'phone_number': s.phoneNumber,
      'otp_verified': s.otpVerified,
      'otp_attempts_today': s.otpAttemptsToday,
      'register_lookups_today': s.registerLookupsToday,
      'objections_today': s.objectionsToday,
    }),
  );

  Future<void> clear() async => _store.delete(key: 'citizen_anon_session');
}

// JSON helpers to avoid import noise
String _encode(Map<String, dynamic> m) {
  // Use dart:convert via toString + manual for compactness
  return _simpleJsonEncode(m);
}

Map<String, dynamic> _decode(String s) {
  return _simpleJsonDecode(s);
}

// Minimal JSON encoder/decoder to avoid dart:convert import in this file
String _simpleJsonEncode(Map<String, dynamic> m) {
  final entries = m.entries.map((e) {
    final v = e.value;
    final jsonV = v == null
        ? 'null'
        : v is String
            ? '"${v.replaceAll('\\', '\\\\').replaceAll('"', '\\"')}"'
            : v is bool
                ? '$v'
                : v is num
                    ? '$v'
                    : '"$v"';
    return '"${e.key}":$jsonV';
  });
  return '{${entries.join(',')}}';
}

Map<String, dynamic> _simpleJsonDecode(String s) {
  // Hand-rolled minimal decode for our flat string/number/bool maps.
  final result = <String, dynamic>{};
  final re = RegExp(r'"([a-z_]+)":(?:"((?:[^"\\]|\\.)*)"|([0-9]+)|(true|false|null))');
  for (final m in re.allMatches(s)) {
    final key = m.group(1)!;
    final strVal = m.group(2);
    final numVal = m.group(3);
    final litVal = m.group(4);
    if (strVal != null) {
      result[key] = strVal.replaceAll(r'\"', '"').replaceAll(r'\\', r'\');
    } else if (numVal != null) {
      result[key] = int.parse(numVal);
    } else if (litVal != null) {
      result[key] = litVal == 'true' ? true : (litVal == 'null' ? null : false);
    }
  }
  return result;
}
