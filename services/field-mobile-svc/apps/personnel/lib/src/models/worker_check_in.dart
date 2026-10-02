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

/// A recorded check-in, matching the dispatch-api `WorkerCheckIn` (camelCase).
class WorkerCheckIn {
  final String id;
  final String workerId;
  final String stationCode;
  final String electionId;
  final String biometricHash;
  final double gpsLat;
  final double gpsLon;
  final double gpsAccuracyM;
  final DateTime checkInAt;
  final String signature;

  const WorkerCheckIn({
    required this.id,
    required this.workerId,
    required this.stationCode,
    required this.electionId,
    required this.biometricHash,
    required this.gpsLat,
    required this.gpsLon,
    required this.gpsAccuracyM,
    required this.checkInAt,
    required this.signature,
  });

  factory WorkerCheckIn.fromJson(Map<String, dynamic> j) => WorkerCheckIn(
        id: j['id'] as String,
        workerId: j['workerId'] as String,
        stationCode: j['stationCode'] as String,
        electionId: j['electionId'] as String,
        biometricHash: j['biometricHash'] as String,
        gpsLat: (j['gpsLat'] as num).toDouble(),
        gpsLon: (j['gpsLon'] as num).toDouble(),
        gpsAccuracyM: (j['gpsAccuracyM'] as num).toDouble(),
        checkInAt: DateTime.parse(j['checkInAt'] as String),
        signature: j['signature'] as String,
      );
}
