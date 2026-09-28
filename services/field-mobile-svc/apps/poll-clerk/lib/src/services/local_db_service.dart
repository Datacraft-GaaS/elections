// Local storage via Isar (ADR-0021).
//
// The encryption key is held in `flutter_secure_storage`.
//
// ADR-0021 names "Isar + SQLCipher" for structured data. The declared
// dependency is `isar: ^3.1.0`, whose `Isar.open` has no `engineOptions` and
// no encryption parameter: SQLCipher is not part of that build, so the
// `.isar` file on disk is NOT encrypted at rest. This is a real gap against
// the ADR, not an oversight in this file, and closing it means either a
// build of Isar that ships SQLCipher or a different store. Flagged rather than
// papered over; see the note on [encryptionAtRest].
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:isar/isar.dart';
import 'package:path_provider/path_provider.dart';

class LocalDbService {
  LocalDbService({this.schemas = const []});

  /// Collection schemas to register. Empty until the collections in `models/`
  /// are generated; `Isar.open` rejects an empty schema list, so an empty list
  /// leaves the database closed rather than opening a database with no tables.
  final List<CollectionSchema<dynamic>> schemas;

  bool _initialized = false;
  Isar? _db;
  final FlutterSecureStorage _store = const FlutterSecureStorage();

  /// Whether the store on disk is encrypted. Always false for this build; see
  /// the file header.
  static const bool encryptionAtRest = false;

  Future<void> init(String encryptionKey) async {
    if (_initialized && _db != null) return;
    if (schemas.isEmpty) {
      // Not an error: no collections are registered yet. Callers check
      // `isInitialized` before using the handle.
      return;
    }
    final key = await _store.read(key: 'db_key') ?? encryptionKey;
    await _store.write(key: 'db_key', value: key);
    final dir = (await getApplicationDocumentsDirectory()).path;
    _db = await Isar.open(schemas, directory: dir);
    _initialized = true;
  }

  bool get isInitialized => _initialized && _db != null;

  Isar? get db => _db;
}
