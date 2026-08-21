enum CountersignType { verified, disputed }

class AgentCountersignature {
	final String id;
	final String formId;
	final String formType;
	final String agentId;
	final String stationCode;
	final DateTime countersignAt;
	final CountersignType countersignType;
	final String? disputeReason;
	final String signature;

	const AgentCountersignature({
		required this.id,
		required this.formId,
		required this.formType,
		required this.agentId,
		required this.stationCode,
		required this.countersignAt,
		required this.countersignType,
		this.disputeReason,
		required this.signature,
	});

	Map<String, dynamic> toJson() => {
		'id': id,
		'formId': formId,
		'formType': formType,
		'agentId': agentId,
		'stationCode': stationCode,
		'countersignAt': countersignAt.toIso8601String(),
		'countersignType': countersignType.name,
		'disputeReason': disputeReason,
		'signature': signature,
	};
}
