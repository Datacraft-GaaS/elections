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

/// Assignment lifecycle state, matching the dispatch-api `AssignmentStatus`
/// (`SCREAMING_SNAKE_CASE`).
enum AssignmentStatus {
  pending,
  accepted,
  declined,
  reassigned;

  static AssignmentStatus fromApi(String value) => switch (value) {
        'PENDING' => AssignmentStatus.pending,
        'ACCEPTED' => AssignmentStatus.accepted,
        'DECLINED' => AssignmentStatus.declined,
        'REASSIGNED' => AssignmentStatus.reassigned,
        _ => AssignmentStatus.pending,
      };
}

/// A worker's dispatch assignment, matching the dispatch-api `WorkerAssignment`
/// (camelCase).
class WorkerAssignment {
  final String id;
  final String workerId;
  final String stationCode;
  final String electionId;
  final String role;
  final DateTime reportTime;
  final String supervisorId;
  final AssignmentStatus status;
  final DateTime? acceptedAt;

  const WorkerAssignment({
    required this.id,
    required this.workerId,
    required this.stationCode,
    required this.electionId,
    required this.role,
    required this.reportTime,
    required this.supervisorId,
    required this.status,
    this.acceptedAt,
  });

  factory WorkerAssignment.fromJson(Map<String, dynamic> j) => WorkerAssignment(
        id: j['id'] as String,
        workerId: j['workerId'] as String,
        stationCode: j['stationCode'] as String,
        electionId: j['electionId'] as String,
        role: j['role'] as String,
        reportTime: DateTime.parse(j['reportTime'] as String),
        supervisorId: j['supervisorId'] as String,
        status: AssignmentStatus.fromApi(j['status'] as String),
        acceptedAt: j['acceptedAt'] == null
            ? null
            : DateTime.parse(j['acceptedAt'] as String),
      );
}
