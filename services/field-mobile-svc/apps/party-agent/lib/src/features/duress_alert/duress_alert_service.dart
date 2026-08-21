// Silent duress alert service stub.
// In production, triggers a silent push to monitoring backend
// without revealing the action in the UI.
class DuressAlertService {
	Future<void> sendSilentAlert({
		required String agentId,
		required String stationCode,
	}) async {
		// Stub: real implementation sends an encrypted beacon via mTLS
	}
}
