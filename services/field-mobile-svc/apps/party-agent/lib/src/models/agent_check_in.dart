class AgentCheckIn {
	final String id;
	final String agentId;
	final String stationCode;
	final String electionId;
	final String biometricAttestationHash;
	final double gpsLat;
	final double gpsLon;
	final DateTime checkInAt;
	final String signature;

	const AgentCheckIn({
		required this.id,
		required this.agentId,
		required this.stationCode,
		required this.electionId,
		required this.biometricAttestationHash,
		required this.gpsLat,
		required this.gpsLon,
		required this.checkInAt,
		required this.signature,
	});

	Map<String, dynamic> toJson() => {
		'id': id,
		'agentId': agentId,
		'stationCode': stationCode,
		'electionId': electionId,
		'biometricAttestationHash': biometricAttestationHash,
		'gpsLat': gpsLat,
		'gpsLon': gpsLon,
		'checkInAt': checkInAt.toIso8601String(),
		'signature': signature,
	};
}
