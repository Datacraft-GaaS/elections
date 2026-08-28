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
