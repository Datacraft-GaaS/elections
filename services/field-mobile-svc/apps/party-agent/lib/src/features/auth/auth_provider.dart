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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

import '../../services/attestation_service.dart';

enum AuthStatus { unauthenticated, authenticated, loading }

class AuthState {
	final AuthStatus status;
	final String? agentId;
	final String? error;

	const AuthState({
		this.status = AuthStatus.unauthenticated,
		this.agentId,
		this.error,
	});

	AuthState copyWith({
		AuthStatus? status,
		String? agentId,
		String? error,
	}) => AuthState(
		status: status ?? this.status,
		agentId: agentId ?? this.agentId,
		error: error ?? this.error,
	);
}

class AuthNotifier extends StateNotifier<AuthState> {
	AuthNotifier(this._attestation) : super(const AuthState());

	final AttestationService _attestation;

	Future<void> login(String agentId, String pin) async {
		state = state.copyWith(status: AuthStatus.loading);

		// ADR-0021: hard-fail MDM attestation before any auth.
		final AttestationResult attestation;
		try {
			attestation = await _attestation.attest();
		} catch (e) {
			state = AuthState(
				status: AuthStatus.unauthenticated,
				error: 'Device attestation failed: $e',
			);
			return;
		}
		if (!attestation.passed) {
			final reason = attestation.failureReason;
			state = AuthState(
				status: AuthStatus.unauthenticated,
				error: reason == null
					? 'Device attestation rejected'
					: 'Device attestation rejected: $reason',
			);
			return;
		}

		state = AuthState(status: AuthStatus.authenticated, agentId: agentId);
	}

	void logout() {
		state = const AuthState();
	}
}

// The stub is software-tier, so attestation fails closed in development rather
// than passing on a machine with no hardware attestation behind it.
final attestationServiceProvider = Provider<AttestationService>(
	(ref) => AttestationService(client: EelAttestationClientStub()),
);

final authProvider = StateNotifierProvider<AuthNotifier, AuthState>(
	(ref) => AuthNotifier(ref.watch(attestationServiceProvider)),
);