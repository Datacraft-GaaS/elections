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

/// Worker presence status. Serialises to the dispatch-api's
/// `SCREAMING_SNAKE_CASE` wire values.
enum WorkerStatus {
  onStation,
  onBreak,
  absent;

  String get apiValue => switch (this) {
        WorkerStatus.onStation => 'ON_STATION',
        WorkerStatus.onBreak => 'BREAK',
        WorkerStatus.absent => 'ABSENT',
      };

  static WorkerStatus fromApi(String value) => switch (value) {
        'ON_STATION' => WorkerStatus.onStation,
        'BREAK' => WorkerStatus.onBreak,
        'ABSENT' => WorkerStatus.absent,
        _ => WorkerStatus.onStation,
      };
}

/// A recorded status ping, matching the dispatch-api `WorkerStatusPing`.
class WorkerStatusPing {
  final String id;
  final String workerId;
  final String stationCode;
  final WorkerStatus status;
  final String? reason;
  final DateTime pingedAt;

  const WorkerStatusPing({
    required this.id,
    required this.workerId,
    required this.stationCode,
    required this.status,
    this.reason,
    required this.pingedAt,
  });

  factory WorkerStatusPing.fromJson(Map<String, dynamic> j) => WorkerStatusPing(
        id: j['id'] as String,
        workerId: j['workerId'] as String,
        stationCode: j['stationCode'] as String,
        status: WorkerStatus.fromApi(j['status'] as String),
        reason: j['reason'] as String?,
        pingedAt: DateTime.parse(j['pingedAt'] as String),
      );
}
