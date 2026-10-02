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

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:flutter/foundation.dart';

/// Session state machine (plan §2.4 auth step):
/// unauthenticated → authenticating → stepUp(biometric+PIN) → authenticated
/// → lockedOut(5 PIN fails, 30 min, persisted, monotonic).
enum SessionPhase {
  unauthenticated,
  authenticating,
  awaitingStepUp,
  active,
  lockedOut,
  expired,
}

@immutable
class SessionState {
  const SessionState({
    required this.phase,
    this.userRole,
    this.userId,
    this.accessToken,
    this.refreshToken,
    this.expiresAt,
    this.attestationBound = false,
    this.pinFailures = 0,
    this.lockedUntil,
  });

  final SessionPhase phase;

  /// SpiceDB-declared role from token (plan §2.4 roleRoute).
  final String? userRole;
  final String? userId;

  /// App session JWT (5-min TTL + refresh; backend rejects attestation
  /// mismatch when [attestationBound] is false).
  final String? accessToken;
  final String? refreshToken;
  final DateTime? expiresAt;
  final bool attestationBound;

  final int pinFailures;
  final DateTime? lockedUntil;

  bool get isAuthenticated => phase == SessionPhase.active;
  bool get isLockedOut => phase == SessionPhase.lockedOut;
  bool get needsRefresh =>
      isAuthenticated && expiresAt != null && DateTime.now().isAfter(expiresAt!);

  SessionState copyWith({
    SessionPhase? phase,
    String? userRole,
    String? userId,
    String? accessToken,
    String? refreshToken,
    DateTime? expiresAt,
    bool? attestationBound,
    int? pinFailures,
    DateTime? lockedUntil,
    bool clearLock = false,
  }) =>
      SessionState(
        phase: phase ?? this.phase,
        userRole: userRole ?? this.userRole,
        userId: userId ?? this.userId,
        accessToken: accessToken ?? this.accessToken,
        refreshToken: refreshToken ?? this.refreshToken,
        expiresAt: expiresAt ?? this.expiresAt,
        attestationBound: attestationBound ?? this.attestationBound,
        pinFailures: pinFailures ?? this.pinFailures,
        lockedUntil: clearLock ? null : (lockedUntil ?? this.lockedUntil),
      );
}

/// PIN lockout policy (plan §2.4): 5 fails → 30-min lockout.
const pinMaxFailures = 5;
const pinLockoutDuration = Duration(minutes: 30);

/// Reducer: pure transitions, persisted lockout handled by the caller.
SessionState reducePinFailure(SessionState state, DateTime Function() now) {
  final failures = state.pinFailures + 1;
  if (failures >= pinMaxFailures) {
    return state.copyWith(
      phase: SessionPhase.lockedOut,
      pinFailures: failures,
      lockedUntil: now().add(pinLockoutDuration),
    );
  }
  return state.copyWith(phase: SessionPhase.awaitingStepUp, pinFailures: failures);
}

SessionState reducePinSuccess(SessionState state) => state.copyWith(
      phase: SessionPhase.active,
      pinFailures: 0,
      clearLock: true,
    );

bool isLockoutActive(SessionState state, DateTime Function() now) {
  final until = state.lockedUntil;
  return until != null && now().isBefore(until);
}

/// OIDC Authorization Code + PKCE flow abstraction (plan §2.4). Apps bind
/// oauth2_client with Keycloak realm `eel`; tests bind a fake.
abstract class OidcFlow {
  /// Returns (accessToken, refreshToken, expiresAt, role, userId).
  Future<({
    String accessToken,
    String refreshToken,
    DateTime expiresAt,
    String? role,
    String? userId,
  })> authorize({
    required List<String> scopes,
    required String attestationVerdict,
  });

  Future<({
    String accessToken,
    String refreshToken,
    DateTime expiresAt,
  })> refresh(String refreshToken);
}

/// Biometric + PIN step-up abstraction (plan §2.4): platform biometric
/// (`local_auth`) + 6-digit PIN.
abstract class StepUpGate {
  Future<bool> canUseBiometrics();
  Future<bool> authenticateBiometric({required String reason});
  Future<bool> verifyPin(String pin);
}