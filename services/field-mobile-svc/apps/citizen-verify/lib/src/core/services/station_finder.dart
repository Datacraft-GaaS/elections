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

/// Station finder (M13 FR-C4) — GPS + search, offline map pin.
import 'package:dio/dio.dart';

class StationFinderService {
  final Dio _dio;
  StationFinderService({Dio? dio}) : _dio = dio ?? Dio(BaseOptions(baseUrl: 'https://api.eel.local'));

  /// Search stations by county/constituency/ward.
  Future<List<StationInfo>> searchStations({String? query, String? county, String? ward}) async {
    try {
      final r = await _dio.get('/api/v1/mobile/bff/citizen-verify/stations',
        queryParameters: {
          if (query != null) 'q': query,
          if (county != null) 'county': county,
          if (ward != null) 'ward': ward,
        },
      );
      if (r.statusCode == 200) {
        final list = r.data['stations'] as List?;
        return list?.map((j) => StationInfo.fromJson(j)).toList() ?? [];
      }
      return [];
    } catch (_) { return []; }
  }

  /// Find nearest station to GPS coordinates.
  Future<StationInfo?> nearestStation({required double lat, required double lon}) async {
    try {
      final r = await _dio.get('/api/v1/mobile/bff/citizen-verify/stations/nearest',
        queryParameters: {'lat': lat.toString(), 'lon': lon.toString()},
      );
      if (r.statusCode == 200 && r.data != null) return StationInfo.fromJson(r.data);
      return null;
    } catch (_) { return null; }
  }
}

class StationInfo {
  final String stationCode;
  final String? stationName;
  final String? countyName;
  final String? wardName;
  final String? constituencyName;
  final double? lat;
  final double? lon;
  final int? queueDepth;
  final String? openStatus;

  const StationInfo({
    required this.stationCode,
    this.stationName,
    this.countyName,
    this.wardName,
    this.constituencyName,
    this.lat,
    this.lon,
    this.queueDepth,
    this.openStatus,
  });

  factory StationInfo.fromJson(Map<String, dynamic> j) => StationInfo(
    stationCode: j['station_code'] as String,
    stationName: j['station_name'] as String?,
    countyName: j['county_name'] as String?,
    wardName: j['ward_name'] as String?,
    constituencyName: j['constituency_name'] as String?,
    lat: (j['lat'] as num?)?.toDouble(),
    lon: (j['lon'] as num?)?.toDouble(),
    queueDepth: (j['queue_depth'] as num?)?.toInt(),
    openStatus: j['open_status'] as String?,
  );
}
