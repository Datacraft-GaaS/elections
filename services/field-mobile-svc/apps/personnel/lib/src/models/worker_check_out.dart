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
}
