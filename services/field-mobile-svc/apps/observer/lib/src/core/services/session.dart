/// Observer session - roles: OBSERVER, OBSERVER_CHIEF
/// Per plan.md M34: accreditation check, observations, station verification
library;

import 'dart:convert';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:local_auth/local_auth.dart';
import 'package:uuid/uuid.dart';

const maxPinAttempts = 5;
const lockoutMinutes = 30;

enum SessionPhase { unauthenticated, biometricStepUp, pinStepUp, authenticated, sessionActive, locked, expired }

class SessionState {
  final SessionPhase phase;
  final String? accessToken;
  final String? refreshToken;
  final String? userId;
  final String? role; // OBSERVER or OBSERVER_CHIEF
  final String? observerId;
  final String? accreditationPassId;
  final int pinFailures;
  final DateTime? lockedUntil;
  final DateTime expiresAt;
  final String sessionId;

  const SessionState({
    this.phase = SessionPhase.unauthenticated,
    this.accessToken,
    this.refreshToken,
    this.userId,
    this.role,
    this.observerId,
    this.accreditationPassId,
    this.pinFailures = 0,
    this.lockedUntil,
    required this.expiresAt,
    required this.sessionId,
  });

  factory SessionState.initial() => SessionState(sessionId: const Uuid().v7(), expiresAt: DateTime.now());

  factory SessionState.authenticated({required String userId, String? role, String? observerId, String? accreditationPassId, required String accessToken, required String refreshToken}) =>
    SessionState(
      phase: SessionPhase.authenticated,
      accessToken: accessToken,
      refreshToken: refreshToken,
      userId: userId,
      role: role ?? 'observer',
      observerId: observerId,
      accreditationPassId: accreditationPassId,
      sessionId: const Uuid().v7(),
      expiresAt: DateTime.now().add(const Duration(hours: 8)), // Longer for observers (12h shifts)
    );

  SessionState afterPinFailure() {
    final nf = pinFailures + 1;
    return SessionState(
      phase: nf >= maxPinAttempts ? SessionPhase.locked : phase,
      accessToken: accessToken,
      refreshToken: refreshToken,
      userId: userId,
      role: role,
      observerId: observerId,
      accreditationPassId: accreditationPassId,
      pinFailures: nf,
      lockedUntil: nf >= maxPinAttempts ? DateTime.now().add(const Duration(minutes: lockoutMinutes)) : null,
      expiresAt: expiresAt,
      sessionId: sessionId,
    );
  }

  bool get isLocked {
    if (lockedUntil == null) return false;
    return DateTime.now().isBefore(lockedUntil!);
  }

  SessionState copyWith({
    SessionPhase? phase, String? accessToken, String? refreshToken, String? userId,
    String? role, String? observerId, String? accreditationPassId, int? pinFailures,
    DateTime? lockedUntil, DateTime? expiresAt, String? sessionId,
  }) => SessionState(
    phase: phase ?? this.phase,
    accessToken: accessToken ?? this.accessToken,
    refreshToken: refreshToken ?? this.refreshToken,
    userId: userId ?? this.userId,
    role: role ?? this.role,
    observerId: observerId ?? this.observerId,
    accreditationPassId: accreditationPassId ?? this.accreditationPassId,
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
      final d = await _store.read(key: 'observer_session');
      if (d == null) return SessionState.initial();
      final data = jsonDecode(d) as Map<String, dynamic>;
      return SessionState(
        phase: SessionPhase.sessionActive,
        accessToken: data['access_token'] as String?,
        refreshToken: data['refresh_token'] as String?,
        userId: data['user_id'] as String?,
        role: data['role'] as String?,
        observerId: data['observer_id'] as String?,
        accreditationPassId: data['accreditation_pass_id'] as String?,
        pinFailures: data['pin_failures'] as int? ?? 0,
        lockedUntil: data['locked_until'] != null ? DateTime.parse(data['locked_until']) : null,
        expiresAt: DateTime.parse(data['expires_at']),
        sessionId: data['session_id'] as String? ?? const Uuid().v7(),
      );
    } catch (_) { return SessionState.initial(); }
  }

  Future<void> save(SessionState s) async => _store.write(key: 'observer_session', value: jsonEncode({
    'access_token': s.accessToken,
    'refresh_token': s.refreshToken,
    'user_id': s.userId,
    'role': s.role,
    'observer_id': s.observerId,
    'accreditation_pass_id': s.accreditationPassId,
    'pin_failures': s.pinFailures,
    'locked_until': s.lockedUntil?.toIso8601String(),
    'expires_at': s.expiresAt.toIso8601String(),
    'session_id': s.sessionId,
  }));

  Future<void> clear() async => await _store.delete(key: 'observer_session');
  Future<bool> canCheckBiometrics() async => await _auth.canCheckBiometrics || await _auth.isDeviceSupported();
  Future<bool> authBio() async => await _auth.authenticate(localizedReason: 'Observer identity', options: const AuthenticationOptions(biometricOnly: true));
}
