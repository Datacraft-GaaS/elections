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
}
