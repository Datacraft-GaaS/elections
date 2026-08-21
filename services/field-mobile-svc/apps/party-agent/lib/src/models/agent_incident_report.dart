class AgentIncidentReport {
	final String id;
	final String agentId;
	final String stationCode;
	final String incidentType;
	final String description;
	final List<String> evidenceBlobIds;
	final double gpsLat;
	final double gpsLon;
	final DateTime submittedAt;
	final String signature;

	const AgentIncidentReport({
		required this.id,
		required this.agentId,
		required this.stationCode,
		required this.incidentType,
		required this.description,
		required this.evidenceBlobIds,
		required this.gpsLat,
		required this.gpsLon,
		required this.submittedAt,
		required this.signature,
	});

	Map<String, dynamic> toJson() => {
		'id': id,
		'agentId': agentId,
		'stationCode': stationCode,
		'incidentType': incidentType,
		'description': description,
		'evidenceBlobIds': evidenceBlobIds,
		'gpsLat': gpsLat,
		'gpsLon': gpsLon,
		'submittedAt': submittedAt.toIso8601String(),
		'signature': signature,
	};
}
