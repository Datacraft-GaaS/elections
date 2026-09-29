/// DualBiometricCapture — 2-factor enrollment / verification.
/// Captures BOTH fingerprint (FAP60) AND iris (AntIris) in a single flow.
/// Logs every capture to AuditService with all 5 axes: where/when/who/why/how.
import 'dart:convert';
import 'package:eel_core_flutter/eel_core_flutter.dart';
import 'package:eel_core_flutter/eel_core_flutter.dart';
import 'package:eel_kit_sdk/src/bundle_security.dart';
import 'package:crypto/crypto.dart' as crypto;
import 'package:uuid/uuid.dart';
import 'package:eel_core_flutter/eel_core_flutter.dart';
import '../fap60/fap60_driver.dart';
import 'bvi_driver.dart';
// AntIrisResult only; this package has its own AntIrisDriver, which shadows
// the bvi-kit one, so the imported driver is hidden to keep it unambiguous.
import 'package:eel_bvi_kit/src/core/drivers/antiris/antiris_driver.dart'
    hide AntIrisDriver;

/// Result of a dual biometric capture (fingerprint + iris).
class DualBiometricResult {
	final FingerprintResult? fp;
	final IrisResult? iris;
	final bool isValid;        // Both must be valid for 2-factor
	final String? auditId;     // UUID7 of the audit entry
	final String? proofHash;   // SHA-256 of combined canonical payload
	final String? error;

	const DualBiometricResult({
		this.fp,
		this.iris,
		required this.isValid,
		this.auditId,
		this.proofHash,
		this.error,
	});

	factory DualBiometricResult.invalid(String error) => DualBiometricResult(isValid: false, error: error);
}

/// Dual biometric capture session.
class DualBiometricCapture {
	final Fap60Driver _fp;
	final AntIrisDriver _iris;
	final AuditService _audit;
	final LocationService _location;
	late final BiometricSubmissionOutbox _outbox;

	DualBiometricCapture({
		BiometricSubmissionOutbox? outbox,
		Fap60Driver? fp,
		AntIrisDriver? iris,
		required AuditService audit,
		LocationService? location,
	})  : _fp = fp ?? Fap60Driver(),
			_iris = iris ?? AntIrisDriver(),
			_audit = audit,
			_location = location ?? LocationService() {
		// The outbox runs local dedup, which means matching, so it needs the
		// drivers' match functions. They cannot be passed in an initialiser
		// list because _fp and _iris are being assigned in the same list.
		_outbox = outbox ??
			BiometricSubmissionOutbox(
				matchFingerprint: _fp.match,
				matchIris: _iris.match,
			);
	}

	/// 2-factor enrollment: capture fingerprint THEN iris.
	/// Returns DualBiometricResult with auditId, proofHash, and both biometrics.
	Future<DualBiometricResult> enroll({
		required String actorId,
		required String actorRole,
		required String target,        // station_id or form_id
		required String reason,        // Justification (FR-010 etc.)
		required String sessionId,
		required String enrolledFpTemplate,
		required String enrolledIrisTemplate,
		bool isOffline = false,
	}) async {
		final auditId = const Uuid().v7();
		final whenUtc = DateTime.now().toUtc().toIso8601String();
		final whereGps = _location.capture();

		// Step 1: Fingerprint capture
		final fpResult = await _fp.captureFingerprint(position: 'LEFT_INDEX');
		if (!fpResult.isValid) {
			return _logAndReturn(
				actorId, actorRole, 'enroll_fingerprint', target, reason,
				whereGps, whenUtc, 'biometric_attestation_fingerprint',
				fpResult.templateBase64 ?? '', auditId, sessionId, isOffline,
				DualBiometricResult.invalid('fingerprint_capture_failed: ${fpResult.error}'),
			);
		}

		// Step 2: Iris capture
		final irisResult = await _iris.captureIris();
		if (!irisResult.isValid) {
			return _logAndReturn(
				actorId, actorRole, 'enroll_iris', target, reason,
				whereGps, whenUtc, 'biometric_attestation_iris',
				irisResult.isoBase64 ?? '', auditId, sessionId, isOffline,
				DualBiometricResult.invalid('iris_capture_failed: ${irisResult.error}'),
			);
		}

		// Step 3: Both valid — compute combined proof hash
		final proofPayload = canonicalPayload(
			auditId: auditId,
			fpTemplate: fpResult.templateBase64 ?? '',
			irisTemplate: irisResult.isoBase64 ?? '',
			whenUtc: whenUtc,
			whereGps: whereGps,
			actorId: actorId,
		);
		final proofHash = sha256(proofPayload);

		// Step 4: Sign with HSM / device key (Ed25519)
		final signature = await _sign(proofPayload);

		// Step 5: Log full audit entry
		await _audit.log(
			actorId: actorId,
			actorRole: actorRole,
			action: 'enroll_dual_biometric',
			target: target,
			reason: reason,
			whereGps: whereGps,
			howMethod: 'fingerprint_fap60_plus_iris_antiris',
			howProofHash: proofHash,
			howSignature: signature,
			isOffline: isOffline,
		);

		// ── Submit to outbox: local dedup + operator-controlled upload ──
		final outboxResult = await _outbox.enqueue(
			actorId: actorId,
			actorRole: actorRole,
			fpTemplate: fpResult.templateBase64,
			irisTemplate: irisResult.isoBase64,
			fpQualityScore: fpResult.qualityScore,
			irisQualityScore: irisResult.quality ?? 0,
			whereGps: whereGps,
			auditId: auditId,
			proofHash: proofHash,
			signature: signature,
		);

		// Local dedup ran but its verdict was discarded, so a duplicate capture
		// was reported as a fresh valid enrollment. A duplicate is not a new
		// enrollment: report it as such, keeping the captured biometrics and
		// pointing at the submission that already exists.
		if (outboxResult.dedupResult.isDuplicate) {
			return DualBiometricResult(
				fp: FingerprintResult.fromFap60(fpResult),
				iris: IrisResult.fromAntIris(irisResult),
				isValid: false,
				auditId: auditId,
				proofHash: proofHash,
				error: 'duplicate_submission:'
					'${outboxResult.dedupResult.matchType.name}'
					'${outboxResult.dedupResult.existingSubmissionId == null ? '' : ' '
						'existing=${outboxResult.dedupResult.existingSubmissionId}'}',
			);
		}

		return DualBiometricResult(
			fp: FingerprintResult.fromFap60(fpResult),
			iris: IrisResult.fromAntIris(irisResult),
			isValid: true,
			auditId: auditId,
			proofHash: proofHash,
		);
	}

	/// 2-factor verification: match captured biometric against enrolled template.
	Future<DualBiometricResult> verify({
		required String actorId,
		required String actorRole,
		required String target,
		required String reason,
		required String sessionId,
		required String enrolledFpTemplate,
		required String enrolledIrisTemplate,
		bool isOffline = false,
	}) async {
		final auditId = const Uuid().v7();
		final whenUtc = DateTime.now().toUtc().toIso8601String();
		final whereGps = _location.capture();

		// Capture fingerprint
		final fpResult = await _fp.captureFingerprint();
		if (!fpResult.isValid) {
			return _logAndReturn(
				actorId, actorRole, 'verify_fingerprint', target, reason,
				whereGps, whenUtc, 'biometric_attestation_fingerprint',
				fpResult.templateBase64 ?? '', auditId, sessionId, isOffline,
				DualBiometricResult.invalid('fingerprint_verify_failed: ${fpResult.error}'),
			);
		}

		// Capture iris
		final irisResult = await _iris.captureIris();
		if (!irisResult.isValid) {
			return _logAndReturn(
				actorId, actorRole, 'verify_iris', target, reason,
				whereGps, whenUtc, 'biometric_attestation_iris',
				irisResult.isoBase64 ?? '', auditId, sessionId, isOffline,
				DualBiometricResult.invalid('iris_verify_failed: ${irisResult.error}'),
			);
		}

		// Match: 1:1 fingerprint + 1:1 iris (templates passed in from prior enrollment)
		final fpMatch = await _fp.match(fpResult.templateBase64!, enrolledFpTemplate);
		final irisMatch = await _iris.match(irisResult.isoBase64!, enrolledIrisTemplate);

		if (!fpMatch || !irisMatch) {
			return _logAndReturn(
				actorId, actorRole, 'verify_dual_failed', target, reason,
				whereGps, whenUtc, 'fingerprint_fap60_plus_iris_antiris',
				fpResult.templateBase64 ?? '', auditId, sessionId, isOffline,
				DualBiometricResult.invalid('match_failed: fp=$fpMatch iris=$irisMatch'),
			);
		}

		final proofPayload = canonicalPayload(
			auditId: auditId,
			fpTemplate: fpResult.templateBase64 ?? '',
			irisTemplate: irisResult.isoBase64 ?? '',
			whenUtc: whenUtc,
			whereGps: whereGps,
			actorId: actorId,
		);
		final proofHash = sha256(proofPayload);
		final signature = await _sign(proofPayload);

		await _audit.log(
			actorId: actorId, actorRole: actorRole,
			action: 'verify_dual_biometric',
			target: target, reason: reason,
			whereGps: whereGps,
			howMethod: 'fingerprint_fap60_plus_iris_antiris',
			howProofHash: proofHash, howSignature: signature,
			isOffline: isOffline,
		);

		return DualBiometricResult(
			fp: FingerprintResult.fromFap60(fpResult),
			iris: IrisResult.fromAntIris(irisResult),
			isValid: true, auditId: auditId, proofHash: proofHash,
		);
	}

	DualBiometricResult _logAndReturn(
		String actorId, String actorRole, String action,
		String target, String reason, String whereGps, String whenUtc,
		String method, String proofHash, String auditId, String sessionId,
		bool isOffline, DualBiometricResult result,
	) {
		_audit.log(
			actorId: actorId, actorRole: actorRole, action: action,
			target: target, reason: reason, whereGps: whereGps,
			howMethod: method, howProofHash: sha256(proofHash),
			howSignature: '', isOffline: isOffline,
		);
		return result;
	}

	String canonicalPayload({
		required String auditId,
		required String fpTemplate,
		required String irisTemplate,
		required String whenUtc,
		required String whereGps,
		required String actorId,
	}) =>
		'{"auditId":"$auditId","fp":"$fpTemplate","iris":"$irisTemplate","when":"$whenUtc","where":"$whereGps","actor":"$actorId"}';

	String sha256(String input) {
		final bytes = utf8.encode(input);
		return crypto.sha256.convert(bytes).toString();
	}

	Future<String> _sign(String payload) async {
		final key = await EelSigningKey.loadOrCreate();
		final sig = await key.sign(utf8.encode(payload));
		return sig;
	}
}

// Re-export result types
class FingerprintResult {
	final bool isValid;
	final String? templateBase64;
	final int qualityScore;
	final String? error;
	const FingerprintResult({required this.isValid, this.templateBase64, this.qualityScore = 0, this.error});
	factory FingerprintResult.fromFap60(Fap60Result r) => FingerprintResult(
		isValid: r.isValid, templateBase64: r.templateBase64,
		qualityScore: r.qualityScore, error: r.error,
	);
}

class IrisResult {
	final bool isValid;
	final String? isoBase64;
	final int? quality;
	final String? error;
	const IrisResult({required this.isValid, this.isoBase64, this.quality, this.error});
	factory IrisResult.fromAntIris(AntIrisResult r) => IrisResult(
		isValid: r.isValid, isoBase64: r.isoBase64,
		quality: r.quality, error: r.error,
	);
}
