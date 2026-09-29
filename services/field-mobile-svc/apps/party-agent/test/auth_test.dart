import 'package:flutter_test/flutter_test.dart';
import 'package:eel_party_agent/src/features/auth/auth_provider.dart';
import 'package:eel_party_agent/src/services/attestation_service.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

/// Attestation client that always fails, simulating an unattested device.
class _FailingAttestationClient implements EelAttestationClient {
	@override
	Future<AttestationVerdict> attest() async {
		throw const AttestationUnavailable('integrity check rejected');
	}
}

/// Attestation client that returns a hardware-backed, unexpired verdict.
class _PassingAttestationClient implements EelAttestationClient {
	@override
	Future<AttestationVerdict> attest() async => AttestationVerdict(
		deviceId: 'test-device',
		tier: AttestationTier.tpm,
		expiresAt: DateTime.now().add(const Duration(hours: 1)),
	);
}

class AttestationUnavailable implements Exception {
	const AttestationUnavailable(this.reason);

	final String reason;

	@override
	String toString() => 'AttestationUnavailable: $reason';
}

void main() {
	group('AuthNotifier', () {
		test('login authenticates after successful attestation', () async {
			final notifier = AuthNotifier(
				AttestationService(client: _PassingAttestationClient()),
			);

			await notifier.login('agent-001', '1234');

			expect(notifier.state.status, AuthStatus.authenticated);
			expect(notifier.state.agentId, 'agent-001');
			expect(notifier.state.error, isNull);
		});

		test('login is blocked when attestation is software-tier only', () async {
			// The stub is software-tier: no hardware attestation happened, so the
			// gate must reject it rather than let a dev build authenticate.
			final notifier = AuthNotifier(
				AttestationService(client: EelAttestationClientStub()),
			);

			await notifier.login('agent-001', '1234');

			expect(notifier.state.status, AuthStatus.unauthenticated);
			expect(notifier.state.agentId, isNull);
			expect(notifier.state.error, contains('software-tier'));
		});

		test('login fails closed when attestation fails (ADR-0021)', () async {
			final notifier = AuthNotifier(
				AttestationService(client: _FailingAttestationClient()),
			);

			await notifier.login('agent-001', '1234');

			expect(notifier.state.status, AuthStatus.unauthenticated);
			expect(notifier.state.agentId, isNull);
			expect(notifier.state.error, contains('attestation'));
		});

		test('logout resets to unauthenticated', () async {
			final notifier = AuthNotifier(AttestationService(client: EelAttestationClientStub()));
			await notifier.login('agent-002', '5678');

			notifier.logout();

			expect(notifier.state.status, AuthStatus.unauthenticated);
			expect(notifier.state.agentId, isNull);
		});
	});
}
