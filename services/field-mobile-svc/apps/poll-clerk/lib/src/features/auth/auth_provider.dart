import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

import '../../services/attestation_service.dart';

enum AuthStatus { unauthenticated, authenticated, loading }

class AuthState {
	final AuthStatus status;
	final String? clerkId;
	final String? error;

	const AuthState({
		this.status = AuthStatus.unauthenticated,
		this.clerkId,
		this.error,
	});

	AuthState copyWith({AuthStatus? status, String? clerkId, String? error}) =>
		AuthState(
			status: status ?? this.status,
			clerkId: clerkId ?? this.clerkId,
			error: error ?? this.error,
		);
}

class AuthNotifier extends StateNotifier<AuthState> {
	AuthNotifier(this._attestation) : super(const AuthState());

	final AttestationService _attestation;

	Future<void> login(String clerkId, String pin) async {
		state = state.copyWith(status: AuthStatus.loading);

		// ADR-0021: hard-fail device attestation before any auth. This used to
		// sleep and then set authenticated unconditionally, so any credentials
		// at all produced a signed-in session.
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

		state = AuthState(status: AuthStatus.authenticated, clerkId: clerkId);
	}

	void logout() => state = const AuthState();
}

// The stub is software-tier, so attestation fails closed in development rather
// than letting a device with no hardware attestation behind it sign in.
final attestationServiceProvider = Provider<AttestationService>(
	(ref) => AttestationService(client: EelAttestationClientStub()),
);

final authProvider = StateNotifierProvider<AuthNotifier, AuthState>(
	(ref) => AuthNotifier(ref.watch(attestationServiceProvider)),
);
