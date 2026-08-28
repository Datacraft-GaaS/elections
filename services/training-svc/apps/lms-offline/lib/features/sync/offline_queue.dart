import 'dart:convert';

import 'package:sqflite/sqflite.dart';
import 'package:path/path.dart' as p;

const _kTable = 'offline_assessment_queue';

class OfflineQueueEntry {
  final String id;
  final String workerId;
  final String moduleId;
  final String idempotencyKey;
  final Map<String, String> answers;
  final String status; // pending / synced / failed
  final DateTime createdAt;

  const OfflineQueueEntry({
    required this.id,
    required this.workerId,
    required this.moduleId,
    required this.idempotencyKey,
    required this.answers,
    required this.status,
    required this.createdAt,
  });

  factory OfflineQueueEntry.fromRow(Map<String, dynamic> row) {
    return OfflineQueueEntry(
      id: row['id'] as String,
      workerId: row['worker_id'] as String,
      moduleId: row['module_id'] as String,
      idempotencyKey: row['idempotency_key'] as String,
      answers: Map<String, String>.from(
        jsonDecode(row['answers_json'] as String) as Map,
      ),
      status: row['status'] as String,
      createdAt: DateTime.parse(row['created_at'] as String),
    );
  }

  Map<String, dynamic> toRow() => {
        'id': id,
        'worker_id': workerId,
        'module_id': moduleId,
        'idempotency_key': idempotencyKey,
        'answers_json': jsonEncode(answers),
        'status': status,
        'created_at': createdAt.toIso8601String(),
      };
}

class OfflineQueue {
  Database? _db;

  Future<Database> get _database async {
    _db ??= await _open();
    return _db!;
  }

  Future<Database> _open() async {
    final dbPath = await getDatabasesPath();
    final path = p.join(dbPath, 'lms_offline.db');
    return openDatabase(
      path,
      version: 1,
      onCreate: (db, version) async {
        await db.execute('''
          CREATE TABLE $_kTable (
            id TEXT PRIMARY KEY,
            worker_id TEXT NOT NULL,
            module_id TEXT NOT NULL,
            idempotency_key TEXT NOT NULL UNIQUE,
            answers_json TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL
          )
        ''');
        await db.execute('''
          CREATE TABLE cached_modules (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            target_audience TEXT NOT NULL,
            language TEXT NOT NULL,
            content_markdown TEXT NOT NULL,
            asset_ids_json TEXT NOT NULL,
            pass_mark INTEGER NOT NULL DEFAULT 80,
            module_type TEXT NOT NULL,
            cached_at TEXT NOT NULL
          )
        ''');
      },
    );
  }

  Future<void> insert(OfflineQueueEntry entry) async {
    final db = await _database;
    // idempotency_key is UNIQUE, so insert or ignore on conflict
    await db.insert(
      _kTable,
      entry.toRow(),
      conflictAlgorithm: ConflictAlgorithm.ignore,
    );
  }

  Future<List<OfflineQueueEntry>> getPending() async {
    final db = await _database;
    final rows = await db.query(
      _kTable,
      where: 'status = ?',
      whereArgs: ['pending'],
      orderBy: 'created_at ASC',
    );
    return rows.map(OfflineQueueEntry.fromRow).toList();
  }

  Future<void> markSynced(String idempotencyKey) async {
    final db = await _database;
    await db.update(
      _kTable,
      {'status': 'synced'},
      where: 'idempotency_key = ?',
      whereArgs: [idempotencyKey],
    );
  }

  Future<void> markFailed(String idempotencyKey) async {
    final db = await _database;
    await db.update(
      _kTable,
      {'status': 'failed'},
      where: 'idempotency_key = ?',
      whereArgs: [idempotencyKey],
    );
  }

  Future<bool> existsByIdempotencyKey(String key) async {
    final db = await _database;
    final rows = await db.query(
      _kTable,
      where: 'idempotency_key = ?',
      whereArgs: [key],
      limit: 1,
    );
    return rows.isNotEmpty;
  }

  Future<Database> get rawDb async => _database;

  /// Close the underlying database (used by tests and lifecycle teardown).
  Future<void> close() async {
    final db = _db;
    _db = null;
    await db?.close();
  }
}
