import 'package:flutter_secure_storage/flutter_secure_storage.dart';

/// Tiered secure store.
///
/// Sensitive keys go to flutter_secure_storage (OS keychain/keystore).
/// Bulk encrypted data goes to Isar with a key derived from attestation;
/// that wiring is left for the concrete kit runtime.
class EelSecureStore {
  EelSecureStore({FlutterSecureStorage? storage})
      : _storage = storage ??
            const FlutterSecureStorage(
              aOptions: AndroidOptions(encryptedSharedPreferences: true),
              iOptions:
                  IOSOptions(accessibility: KeychainAccessibility.first_unlock),
            );

  final FlutterSecureStorage _storage;

  Future<void> write(String key, String value) =>
      _storage.write(key: key, value: value);

  Future<String?> read(String key) => _storage.read(key: key);

  Future<void> delete(String key) => _storage.delete(key: key);

  Future<Map<String, String>> readAll() => _storage.readAll();
}
