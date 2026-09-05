/// Encrypted local storage via Isar + SQLCipher (ADR-0021).
/// Uses `flutter_secure_storage` for the encryption key and `isar` for collections.
import 'package:isar/isar.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class LocalDbService {
  bool _initialized = false;
  Isar? _db;
  final FlutterSecureStorage _store = const FlutterSecureStorage();

  Future<void> init(String encryptionKey) async {
    if (_initialized && _db != null) return;
    final key = await _store.read(key: 'db_key') ?? encryptionKey;
    await _store.write(key: 'db_key', value: key);
    // Real: open Isar with SQLCipher engine using `key`.
    // Schema collections are registered in `models/` (see Isar docs M0.2).
    _db = await Isar.open(
      [/* collections registered via schema files */],
      directory: '.',
      engineOptions: IsarEngineOptions(
        encryptionKey: key,
      ),
    );
    _initialized = true;
  }

  bool get isInitialized => _initialized && _db != null;
  Isar? get db => _db;
}
