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
