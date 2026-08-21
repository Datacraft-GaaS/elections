// Isar + SQLCipher stub.
// ADR-0021: encrypted local storage via Isar with SQLCipher engine.
class LocalDbService {
	bool _initialized = false;

	Future<void> init(String encryptionKey) async {
		if (_initialized) return;
		// Stub: real implementation opens Isar with encryption key from
		// flutter_secure_storage and registers all schema collections.
		_initialized = true;
	}

	bool get isInitialized => _initialized;
}
