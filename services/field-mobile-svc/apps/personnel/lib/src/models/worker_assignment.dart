enum AssignmentStatus { pending, confirmed, onStation, completed }

class WorkerAssignment {
  final String id;
  final String workerId;
  final String stationCode;
  final String electionId;
  final String role;
  final DateTime reportTime;
  final String supervisorId;
  final AssignmentStatus status;

  const WorkerAssignment({
    required this.id,
    required this.workerId,
    required this.stationCode,
    required this.electionId,
    required this.role,
    required this.reportTime,
    required this.supervisorId,
    required this.status,
  });
}
