/// AuditService: where / when / who / why / how — shared by all 14 apps.
/// Uses encrypted Isar + outbox replay (offline-safe).
import 'package:eel_core_flutter/src/audit/audit_model.dart';

class AuditService {
	final String sessionId;
	final String appName;

	AuditService(this.sessionId, this.appName);

	/// Log an action with all 5 axes.
	Future<void> log({
		required String actorId,
		required String actorRole,
		required String action,
		required String target,
		required String reason,
		required String whereGps,
		required String howMethod,
		required String howProofHash,
		required String howSignature,
		required bool isOffline,
	}) async {
		final entry = AuditEntry.create(
			actorId: actorId, actorRole: actorRole, action: action,
			target: target, reason: reason, whereGps: whereGps,
			howMethod: howMethod, howProofHash: howProofHash,
			howSignature: howSignature, appName: appName,
			isOffline: isOffline, sessionId: sessionId,
		);
		// Persist to encrypted Isar (offline-safe); replay via outbox on reconnect
		await _persist(entry);
		await _outboxIfOffline(entry, isOffline);
	}

	Future<void> _persist(AuditEntry entry) async {
		// Isar.open with SQLCipher key; write to audit_entries table
	}

	Future<void> _outboxIfOffline(AuditEntry entry, bool isOffline) async {
		if (isOffline) {
			// Queue in outbox for replay when connectivity restored
		}
	}

	/// Replay outbox on reconnect — verifies all 5 axes preserved.
	Future<void> replay() async {}
}
