import 'package:eel_kit_sdk/eel_kit_sdk.dart';

/// MDM device attestation (ADR-0021: hard-fail attestation before any auth).
///
/// Delegates to the kit-sdk attestation module (Play Integrity on Android,
/// App Attest on iOS). Production kits inject a platform-backed
/// [EelAttestationClient] at bootstrap; the SDK software fallback serves
/// development builds.
class AttestationService {
	AttestationService({EelAttestationClient? client})
		: _client = client ?? EelAttestationClientStub();

	final EelAttestationClient _client;

	Future<AttestationResult> attest() async {
		try {
			final verdict = await _client.attest();
			return AttestationResult(passed: true, token: verdict.deviceId);
		} on Exception catch (e) {
			return AttestationResult(passed: false, failureReason: e.toString());
		}
	}
}

class AttestationResult {
	final bool passed;
	final String? token;
	final String? failureReason;

	const AttestationResult({
		required this.passed,
		this.token,
		this.failureReason,
	});
}
