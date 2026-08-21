class IncidentReport {
	final String id;
	final String clerkId;
	final String stationCode;
	final String type;
	final String description;
	final DateTime submittedAt;
	final String signature;

	const IncidentReport({
		required this.id,
		required this.clerkId,
		required this.stationCode,
		required this.type,
		required this.description,
		required this.submittedAt,
		required this.signature,
	});

	Map<String, dynamic> toJson() => {
		'id': id,
		'clerkId': clerkId,
		'stationCode': stationCode,
		'type': type,
		'description': description,
		'submittedAt': submittedAt.toIso8601String(),
		'signature': signature,
	};
}
