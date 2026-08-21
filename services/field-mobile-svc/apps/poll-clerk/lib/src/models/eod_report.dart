class EodReport {
	final String id;
	final String clerkId;
	final String stationCode;
	final int totalVoters;
	final int spoiledBallots;
	final int assistedVoters;
	final DateTime submittedAt;
	final String signature;

	const EodReport({
		required this.id,
		required this.clerkId,
		required this.stationCode,
		required this.totalVoters,
		required this.spoiledBallots,
		required this.assistedVoters,
		required this.submittedAt,
		required this.signature,
	});

	Map<String, dynamic> toJson() => {
		'id': id,
		'clerkId': clerkId,
		'stationCode': stationCode,
		'totalVoters': totalVoters,
		'spoiledBallots': spoiledBallots,
		'assistedVoters': assistedVoters,
		'submittedAt': submittedAt.toIso8601String(),
		'signature': signature,
	};
}
