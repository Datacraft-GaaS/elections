import 'package:flutter_riverpod/flutter_riverpod.dart';

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
	AuthNotifier() : super(const AuthState());

	Future<void> login(String agentId, String pin) async {
		state = state.copyWith(status: AuthStatus.loading);
		// TODO(security): integrate Play Integrity attestation before auth
		await Future.delayed(const Duration(milliseconds: 300));
		state = state.copyWith(
			status: AuthStatus.authenticated,
			agentId: agentId,
		);
	}

	void logout() {
		state = const AuthState();
	}
}

final authProvider = StateNotifierProvider<AuthNotifier, AuthState>(
	(ref) => AuthNotifier(),
);
