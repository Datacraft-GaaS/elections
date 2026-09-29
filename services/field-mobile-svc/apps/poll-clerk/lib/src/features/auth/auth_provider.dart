import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

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
	AuthNotifier() : super(const AuthState());

	Future<void> login(String clerkId, String pin) async {
		state = state.copyWith(status: AuthStatus.loading);
		await Future.delayed(const Duration(milliseconds: 300));
		state = state.copyWith(status: AuthStatus.authenticated, clerkId: clerkId);
	}

	void logout() => state = const AuthState();
}

final authProvider = StateNotifierProvider<AuthNotifier, AuthState>(
	(ref) => AuthNotifier(),
);
