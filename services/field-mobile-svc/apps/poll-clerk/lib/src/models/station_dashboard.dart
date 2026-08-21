class StationDashboard {
	final String stationCode;
	final String assignedClerkId;
	final int voterCount;
	final int processedCount;
	final int incidentCount;
	final DateTime lastSyncAt;

	const StationDashboard({
		required this.stationCode,
		required this.assignedClerkId,
		required this.voterCount,
		required this.processedCount,
		required this.incidentCount,
		required this.lastSyncAt,
	});

	Map<String, dynamic> toJson() => {
		'stationCode': stationCode,
		'assignedClerkId': assignedClerkId,
		'voterCount': voterCount,
		'processedCount': processedCount,
		'incidentCount': incidentCount,
		'lastSyncAt': lastSyncAt.toIso8601String(),
	};
}
