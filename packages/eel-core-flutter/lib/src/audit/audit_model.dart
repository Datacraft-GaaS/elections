/// AuditLog model: where / when / who / why / how — per spec §FR-031.
import 'package:uuid/uuid.dart';

/// The 5 axes: GPS, timestamp, actor identity, justification/reason, method/proof.
class AuditEntry {
	final String auditId;        // UUID7 — per entry
	final String actorId;        // Actor identity (agent, clerk, observer, etc.)
	final String actorRole;      // SpiceDB role or category (CLERK, DPO, KIOSK, etc.)
	final String action;         // Action type: check_in, lookup, incident_create, countersign, etc.
	final String target;         // Target of action: station_id, form_id, incident_id, etc.
	final String reason;         // Justification / reason (FR justification text, dispute reason, etc.)
	final String whereGps;       // GPS coordinate (latitude,longitude,accuracy) or station-bound validation
	final String whenUtc;        // ISO 8601 UTC timestamp
	final String howMethod;      // Method / proof: biometric_attestation_fingerprint, iris, HSM_Ed25519, OTP, PIN, etc.
	final String howProofHash;   // SHA-256 of proof payload (canonical JSON) or biometric hash reference
	final String howSignature;   // Ed25519 signature of canonical audit payload (if signed)
	final String appName;        // Source app: poll-clerk, observer, party-agent, citizen-verify, etc.
	final bool isOffline;        // Whether action occurred while offline
	final String sessionId;      // Session UUID for correlation

	AuditEntry({
		required this.auditId,
		required this.actorId,
		required this.actorRole,
		required this.action,
		required this.target,
		required this.reason,
		required this.whereGps,
		required this.whenUtc,
		required this.howMethod,
		required this.howProofHash,
		required this.howSignature,
		required this.appName,
		required this.isOffline,
		required this.sessionId,
	});

	/// Canonical JSON for signing / verification (RFC 8785 semantics).
	String canonicalPayload() => '{"auditId":"$auditId","actorId":"$actorId","actorRole":"$actorRole","action":"$action","target":"$target","reason":"$reason","whereGps":"$whereGps","whenUtc":"$whenUtc","howMethod":"$howMethod","howProofHash":"$howProofHash","howSignature":"$howSignature","appName":"$appName","isOffline":$isOffline,"sessionId":"$sessionId"}';

	factory AuditEntry.create({
		required String actorId,
		required String actorRole,
		required String action,
		required String target,
		required String reason,
		required String whereGps,
		required String howMethod,
		required String howProofHash,
		required String howSignature,
		required String appName,
		required bool isOffline,
		required String sessionId,
	}) => AuditEntry(
		auditId: const Uuid().v7().toString(),
		actorId: actorId,
		actorRole: actorRole,
		action: action,
		target: target,
		reason: reason,
		whereGps: whereGps,
		whenUtc: DateTime.now().toUtc().toIso8601String(),
		howMethod: howMethod,
		howProofHash: howProofHash,
		howSignature: howSignature,
		appName: appName,
		isOffline: isOffline,
		sessionId: sessionId,
	);
}
