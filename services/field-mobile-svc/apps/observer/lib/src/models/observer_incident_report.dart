class ObserverIncidentReport {
	final String id;
	final String observerId;
	final String stationCode;
	final String category;
	final String description;
	final List<String> evidenceBlobIds;
	final bool isSystemicFlag;
	final DateTime submittedAt;
	final String signature;

	const ObserverIncidentReport({
		required this.id,
		required this.observerId,
		required this.stationCode,
		required this.category,
		required this.description,
		required this.evidenceBlobIds,
		required this.isSystemicFlag,
		required this.submittedAt,
		required this.signature,
	});

	Map<String, dynamic> toJson() => {
		'id': id,
		'observerId': observerId,
		'stationCode': stationCode,
		'category': category,
		'description': description,
		'evidenceBlobIds': evidenceBlobIds,
		'isSystemicFlag': isSystemicFlag,
		'submittedAt': submittedAt.toIso8601String(),
		'signature': signature,
	};
}
