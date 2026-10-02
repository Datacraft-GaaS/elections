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

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:flutter/foundation.dart';

/// Per-app config fetched from `/api/v1/mobile/bff/config` (plan §2.4).
@immutable
class AppConfig {
  const AppConfig({
    required this.appId,
    required this.featureFlags,
    required this.pinSetId,
    required this.versionFloor,
    this.rateLimitPolicy,
    this.stationBinding,
    this.clusterBinding,
    this.fetchedAt,
  });

  final String appId;
  final Map<String, bool> featureFlags;
  final String pinSetId;
  final String versionFloor;
  final Map<String, dynamic>? rateLimitPolicy;
  final String? stationBinding;
  final String? clusterBinding;
  final DateTime? fetchedAt;

  factory AppConfig.fromJson(Map<String, dynamic> json) => AppConfig(
        appId: json['app_id'] as String? ?? json['appId'] as String? ?? '',
        featureFlags:
            (json['feature_flags'] as Map?)?.cast<String, bool>() ?? {},
        pinSetId: json['pin_set_id'] as String? ?? '',
        versionFloor: json['version_floor'] as String? ?? '0.0.0',
        rateLimitPolicy: json['rate_limit_policy'] as Map<String, dynamic>?,
        stationBinding: json['station_binding'] as String?,
        clusterBinding: json['cluster_binding'] as String?,
        fetchedAt: json['fetched_at'] != null
            ? DateTime.parse(json['fetched_at'] as String)
            : null,
      );

  Map<String, dynamic> toJson() => {
        'app_id': appId,
        'feature_flags': featureFlags,
        'pin_set_id': pinSetId,
        'version_floor': versionFloor,
        if (rateLimitPolicy != null) 'rate_limit_policy': rateLimitPolicy,
        if (stationBinding != null) 'station_binding': stationBinding,
        if (clusterBinding != null) 'cluster_binding': clusterBinding,
        if (fetchedAt != null) 'fetched_at': fetchedAt!.toIso8601String(),
      };

  bool flag(String key, {bool fallback = false}) =>
      featureFlags[key] ?? fallback;
}

/// Abstract config transport so the app wires the pinned EelHttpClient.
abstract class ConfigFetcher {
  Future<AppConfig> fetch({required String appId});
}

/// Loads + caches last-good config in secure storage; offline uses cache
/// (plan §2.4 `config` step).
class AppConfigService {
  AppConfigService({
    required ConfigFetcher fetcher,
    required CacheStore cache,
    this.maxStale = const Duration(days: 7),
  })  : _fetcher = fetcher,
        _cache = cache;

  final ConfigFetcher _fetcher;
  final CacheStore _cache;
  final Duration maxStale;

  static const _cacheKeyPrefix = 'eel.config.';

  Future<AppConfig> load({required String appId}) async {
    try {
      final fresh = await _fetcher.fetch(appId: appId);
      await _cache.write(
        '$_cacheKeyPrefix$appId',
        AppConfig.fromJson(fresh.toJson()..['fetched_at'] = DateTime.now().toIso8601String())
            .toJson(),
      );
      return fresh;
    } catch (_) {
      final cached = await _cache.read('$_cacheKeyPrefix$appId');
      if (cached != null) {
        return AppConfig.fromJson(cached);
      }
      rethrow;
    }
  }
}

/// Key-value cache abstraction (EelSecureStore at the app layer).
abstract class CacheStore {
  Future<void> write(String key, Map<String, dynamic> value);
  Future<Map<String, dynamic>?> read(String key);
}

/// Version-floor gate (plan §2.4 version-blocked state; EC-MOB-06).
bool isVersionBlocked({
  required String current,
  required String floor,
}) {
  List<int> parse(String v) => v
      .split('.')
      .map((e) => int.tryParse(e) ?? 0)
      .toList();
  final a = parse(current);
  final b = parse(floor);
  for (var i = 0; i < 3; i++) {
    final av = i < a.length ? a[i] : 0;
    final bv = i < b.length ? b[i] : 0;
    if (av != bv) return av < bv;
  }
  return false;
}