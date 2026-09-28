/// Session/auth for party-agent.
/// 
/// Party agent roles: PARTY_AGENT, PARTY_CHIEF_AGENT
/// OIDC PKCE + local_auth biometric + 6-digit PIN step-up.

import 'dart:convert';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:local_auth/local_auth.dart';
import 'package:uuid/uuid.dart';

/// PIN policy constants
const int maxPinAttempts = 5;
const int lockoutMinutes = 30;

/// Session phase enum
enum SessionPhase {
  unauthenticated,
  biometricStepUp,
  pinStepUp,
  authenticated,
  sessionActive,
  locked,
  expired,
}

/// Immutable session state
class SessionState {
  final SessionPhase phase;
  final String? accessToken;
  final String? refreshToken;
  final String? userId;
  final String? role; // PARTY_AGENT or PARTY_CHIEF_AGENT
  final String? partyCode;
  final int pinFailures;
  final DateTime? lockedUntil;
  final String? attestationVerdict;
  final bool biometricEnabled;
  final DateTime expiresAt;
  final String sessionId;

  const SessionState({
    this.phase = SessionPhase.unauthenticated,
    this.accessToken,
    this.refreshToken,
    this.userId,
    this.role,
    this.partyCode,
    this.pinFailures = 0,
    this.lockedUntil,
    this.attestationVerdict,
    this.biometricEnabled = true,
    required this.expiresAt,
    required this.sessionId,
  });

  factory SessionState.initial() {
    return SessionState(
      sessionId: const Uuid().v7(),
      expiresAt: DateTime.now(),
    );
  }

  factory SessionState.authenticated({
    required String accessToken,
    required String refreshToken,
    required String userId,
    String? role,
    String? partyCode,
    String? attestationVerdict,
  }) {
    return SessionState(
      phase: SessionPhase.authenticated,
      accessToken: accessToken,
      refreshToken: refreshToken,
      userId: userId,
      role: role ?? 'party_agent',
      partyCode: partyCode,
      attestationVerdict: attestationVerdict,
      sessionId: const Uuid().v7(),
      expiresAt: DateTime.now().add(const Duration(hours: 5)),
    );
  }

  SessionState afterPinFailure() {
    final newFailures = pinFailures + 1;
    DateTime? newLockedUntil;
    SessionPhase newPhase = phase;

    if (newFailures >= maxPinAttempts) {
      newLockedUntil = DateTime.now().add(Duration(minutes: lockoutMinutes));
      newPhase = SessionPhase.locked;
    }

    return SessionState(
      phase: newPhase,
      accessToken: accessToken,
      refreshToken: refreshToken,
      userId: userId,
      role: role,
      partyCode: partyCode,
      pinFailures: newFailures,
      lockedUntil: newLockedUntil,
      attestationVerdict: attestationVerdict,
      biometricEnabled: biometricEnabled,
      expiresAt: expiresAt,
      sessionId: sessionId,
    );
  }

  SessionState afterPinSuccess() {
    return SessionState(
      phase: SessionPhase.authenticated,
      accessToken: accessToken,
      refreshToken: refreshToken,
      userId: userId,
      role: role,
      partyCode: partyCode,
      pinFailures: 0,
      lockedUntil: null,
      attestationVerdict: attestationVerdict,
      biometricEnabled: biometricEnabled,
      expiresAt: DateTime.now().add(const Duration(hours: 5)),
      sessionId: sessionId,
    );
  }

  bool get isLocked {
    if (lockedUntil == null) return false;
    return DateTime.now().isBefore(lockedUntil!);
  }

  SessionState expired() {
    return SessionState(
      phase: SessionPhase.expired,
      accessToken: accessToken,
      refreshToken: refreshToken,
      userId: userId,
      role: role,
      partyCode: partyCode,
      pinFailures: pinFailures,
      lockedUntil: lockedUntil,
      attestationVerdict: attestationVerdict,
      biometricEnabled: biometricEnabled,
      expiresAt: DateTime.now(),
      sessionId: sessionId,
    );
  }

  SessionState copyWith({
    SessionPhase? phase,
    String? accessToken,
    String? refreshToken,
    String? userId,
    String? role,
    String? partyCode,
    int? pinFailures,
    DateTime? lockedUntil,
    String? attestationVerdict,
    bool? biometricEnabled,
    DateTime? expiresAt,
    String? sessionId,
  }) {
    return SessionState(
      phase: phase ?? this.phase,
      accessToken: accessToken ?? this.accessToken,
      refreshToken: refreshToken ?? this.refreshToken,
      userId: userId ?? this.userId,
      role: role ?? this.role,
      partyCode: partyCode ?? this.partyCode,
      pinFailures: pinFailures ?? this.pinFailures,
      lockedUntil: lockedUntil ?? this.lockedUntil,
      attestationVerdict: attestationVerdict ?? this.attestationVerdict,
      biometricEnabled: biometricEnabled ?? this.biometricEnabled,
      expiresAt: expiresAt ?? this.expiresAt,
      sessionId: sessionId ?? this.sessionId,
    );
  }
}

/// Session service
class SessionService {
  final FlutterSecureStorage _secureStorage;
  final LocalAuthentication _localAuth;

  SessionService({
    FlutterSecureStorage? secureStorage,
    LocalAuthentication? localAuth,
  })  : _secureStorage = secureStorage ?? const FlutterSecureStorage(),
        _localAuth = localAuth ?? LocalAuthentication();

  Future<SessionState> loadSession() async {
    try {
      final sessionData = await _secureStorage.read(key: 'party_agent_session');
      if (sessionData == null) return SessionState.initial();

      final data = jsonDecode(sessionData) as Map<String, dynamic>;
      final expiresAtStr = data['expires_at'] as String?;
      final expiresAt = expiresAtStr != null ? DateTime.parse(expiresAtStr) : DateTime.now();

      final state = SessionState(
        phase: SessionPhase.sessionActive,
        accessToken: data['access_token'] as String?,
        refreshToken: data['refresh_token'] as String?,
        userId: data['user_id'] as String?,
        role: data['role'] as String?,
        partyCode: data['party_code'] as String?,
        pinFailures: data['pin_failures'] as int? ?? 0,
        lockedUntil: data['locked_until'] != null ? DateTime.parse(data['locked_until']) : null,
        attestationVerdict: data['attestation_verdict'] as String?,
        biometricEnabled: data['biometric_enabled'] == true,
        expiresAt: expiresAt,
        sessionId: data['session_id'] as String? ?? const Uuid().v7(),
      );

      if (DateTime.now().isAfter(expiresAt)) {
        return state.expired();
      }
      return state;
    } catch (e) {
      return SessionState.initial();
    }
  }

  Future<void> saveSession(SessionState session) async {
    try {
      await _secureStorage.write(
        key: 'party_agent_session',
        value: jsonEncode({
          'access_token': session.accessToken,
          'refresh_token': session.refreshToken,
          'user_id': session.userId,
          'role': session.role,
          'party_code': session.partyCode,
          'pin_failures': session.pinFailures,
          'locked_until': session.lockedUntil?.toIso8601String(),
          'attestation_verdict': session.attestationVerdict,
          'biometric_enabled': session.biometricEnabled,
          'expires_at': session.expiresAt.toIso8601String(),
          'session_id': session.sessionId,
        }),
      );
    } catch (e) {
      // Silent fail
    }
  }

  Future<void> clearSession() async {
    try {
      await _secureStorage.delete(key: 'party_agent_session');
      await _secureStorage.delete(key: 'party_agent_pin');
    } catch (e) {}
  }

  Future<bool> canCheckBiometrics() async {
    try {
      return await _localAuth.canCheckBiometrics || await _localAuth.isDeviceSupported();
    } catch (e) {
      return false;
    }
  }

  Future<bool> authenticateWithBiometrics() async {
    try {
      return await _localAuth.authenticate(
        localizedReason: 'Confirm your identity',
        options: const AuthenticationOptions(
          biometricOnly: true,
          stickyAuth: false,
        ),
      );
    } catch (e) {
      return false;
    }
  }

  Future<bool> verifyPin(String enteredPin) async {
    try {
      final storedPin = await _secureStorage.read(key: 'party_agent_pin');
      return storedPin == enteredPin;
    } catch (e) {
      return false;
    }
  }

  Future<void> setPin(String newPin) async {
    await _secureStorage.write(key: 'party_agent_pin', value: newPin);
  }
}
