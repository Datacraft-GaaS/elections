import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:path/path.dart' as p;
import 'package:sqflite/sqflite.dart';

import 'crl_model.dart';

const _kCrlTable = 'crl_cache';
const _kStaleThresholdMinutes = 30;

class CrlRepository {
  final Dio _dio;
  Database? _db;

  CrlRepository({required Dio dio}) : _dio = dio;

  Future<Database> get _database async {
    _db ??= await _openDb();
    return _db!;
  }

  Future<Database> _openDb() async {
    final dbPath = await getDatabasesPath();
    final path = p.join(dbPath, 'accred_scanner.db');
    return openDatabase(
      path,
      version: 1,
      onCreate: (db, version) async {
        await db.execute('''
          CREATE TABLE $_kCrlTable (
            id INTEGER PRIMARY KEY,
            generated_at TEXT NOT NULL,
            revoked_serials_json TEXT NOT NULL,
            cached_at TEXT NOT NULL
          )
        ''');
      },
    );
  }

  Future<CrlModel> fetchCrl() async {
    // M42: CRL endpoint integration — fetch from accredited CA endpoint.
    // Real endpoint: /v1/security/crl (with CA authority verification).
    final resp = await _dio.get<Map<String, dynamic>>('/v1/security/crl',
      options: Options(
        extra: {'require_ca_verification': true},
      ),
    );
    final crl = CrlModel.fromJson(resp.data!);
    await _cacheCrl(crl);
    return crl;
  }

  Future<CrlModel?> getCachedCrl() async {
    final db = await _database;
    final rows = await db.query(
      _kCrlTable,
      orderBy: 'cached_at DESC',
      limit: 1,
    );
    if (rows.isEmpty) return null;
    final row = rows.first;
    return CrlModel(
      generatedAt: DateTime.parse(row['generated_at'] as String),
      revokedSerials: List<String>.from(
        jsonDecode(row['revoked_serials_json'] as String) as List,
      ),
    );
  }

  Future<bool> isCrlStale() async {
    final crl = await getCachedCrl();
    if (crl == null) return true;
    return DateTime.now().difference(crl.generatedAt).inMinutes >
        _kStaleThresholdMinutes;
  }

  Future<bool> isRevoked(String serial) async {
    final crl = await getCachedCrl();
    if (crl == null) return false;
    return crl.revokedSerials.contains(serial);
  }

  Future<void> _cacheCrl(CrlModel crl) async {
    final db = await _database;
    await db.delete(_kCrlTable); // keep only latest
    await db.insert(_kCrlTable, {
      'generated_at': crl.generatedAt.toIso8601String(),
      'revoked_serials_json': jsonEncode(crl.revokedSerials),
      'cached_at': DateTime.now().toIso8601String(),
    });
  }
}
