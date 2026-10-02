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

/// A recorded check-out, matching the dispatch-api `WorkerCheckOut` (camelCase).
class WorkerCheckOut {
  final String id;
  final String workerId;
  final String stationCode;
  final String electionId;
  final double gpsLat;
  final double gpsLon;
  final DateTime checkOutAt;
  final double hoursWorked;
  final String? payrollTriggerId;
  final String signature;

  const WorkerCheckOut({
    required this.id,
    required this.workerId,
    required this.stationCode,
    required this.electionId,
    required this.gpsLat,
    required this.gpsLon,
    required this.checkOutAt,
    required this.hoursWorked,
    this.payrollTriggerId,
    required this.signature,
  });

  factory WorkerCheckOut.fromJson(Map<String, dynamic> j) => WorkerCheckOut(
        id: j['id'] as String,
        workerId: j['workerId'] as String,
        stationCode: j['stationCode'] as String,
        electionId: j['electionId'] as String,
        gpsLat: (j['gpsLat'] as num).toDouble(),
        gpsLon: (j['gpsLon'] as num).toDouble(),
        checkOutAt: DateTime.parse(j['checkOutAt'] as String),
        hoursWorked: (j['hoursWorked'] as num).toDouble(),
        payrollTriggerId: j['payrollTriggerId'] as String?,
        signature: j['signature'] as String,
      );
}
