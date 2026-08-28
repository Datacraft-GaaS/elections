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
