// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import 'dart:convert';

import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:dio/dio.dart';

import '../sync/offline_queue.dart';
import 'module_model.dart';

const _kMaxCached = 5;

class ModuleRepository {
  final Dio _dio;
  final OfflineQueue _queue;

  ModuleRepository({required Dio dio, required OfflineQueue queue})
      : _dio = dio,
        _queue = queue;

  Future<bool> isOffline() async {
    final result = await Connectivity().checkConnectivity();
    return result.contains(ConnectivityResult.none) ||
        result.isEmpty;
  }

  Future<List<CourseModule>> fetchModules() async {
    final resp = await _dio.get<List<dynamic>>('/v1/modules');
    final modules = (resp.data ?? [])
        .map((e) => CourseModule.fromJson(e as Map<String, dynamic>))
        .toList();
    return modules;
  }

  Future<List<CourseModule>> getCachedModules() async {
    final db = await _queue.rawDb;
    final rows = await db.query('cached_modules', orderBy: 'cached_at ASC');
    return rows.map(_rowToModule).toList();
  }

  Future<void> downloadModule(String id) async {
    final db = await _queue.rawDb;
    final existing = await db.query(
      'cached_modules',
      where: 'id = ?',
      whereArgs: [id],
    );
    if (existing.isNotEmpty) return; // already cached

    final resp = await _dio.get<Map<String, dynamic>>('/v1/modules/$id');
    final module = CourseModule.fromJson(resp.data!);

    final cached = await getCachedModules();
    if (cached.length >= _kMaxCached) {
      // Evict oldest by cached_at, never evict in-progress
      final oldest = await db.query(
        'cached_modules',
        orderBy: 'cached_at ASC',
        limit: 1,
      );
      if (oldest.isNotEmpty) {
        await db.delete(
          'cached_modules',
          where: 'id = ?',
          whereArgs: [oldest.first['id']],
        );
      }
    }

    await db.insert('cached_modules', _moduleToRow(module));
  }

  Future<void> evictModule(String id) async {
    final db = await _queue.rawDb;
    await db.delete('cached_modules', where: 'id = ?', whereArgs: [id]);
  }

  CourseModule _rowToModule(Map<String, dynamic> row) {
    return CourseModule(
      id: row['id'] as String,
      title: row['title'] as String,
      targetAudience: row['target_audience'] as String,
      language: row['language'] as String,
      contentMarkdown: row['content_markdown'] as String,
      assetIds: List<String>.from(
        jsonDecode(row['asset_ids_json'] as String) as List,
      ),
      passMark: row['pass_mark'] as int,
      moduleType: ModuleType.values.byName(row['module_type'] as String),
    );
  }

  Map<String, dynamic> _moduleToRow(CourseModule m) {
    return {
      'id': m.id,
      'title': m.title,
      'target_audience': m.targetAudience,
      'language': m.language,
      'content_markdown': m.contentMarkdown,
      'asset_ids_json': jsonEncode(m.assetIds),
      'pass_mark': m.passMark,
      'module_type': m.moduleType.name,
      'cached_at': DateTime.now().toIso8601String(),
    };
  }
}
