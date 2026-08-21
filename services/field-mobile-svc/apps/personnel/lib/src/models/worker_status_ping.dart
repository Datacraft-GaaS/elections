enum WorkerStatus { onStation, onBreak, absent }

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
}
