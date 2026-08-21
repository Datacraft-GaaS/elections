// Play Integrity (Android) and App Attest (iOS) stub.
// ADR-0021: hard-fail MDM attestation before any auth.
class AttestationService {
	Future<AttestationResult> attest() async {
		// Stub: real implementation calls platform channel to invoke
		// Play Integrity API or App Attest, then verifies with backend.
		return const AttestationResult(passed: true, token: 'stub-token');
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
