class ObserverVerificationRecord {
	final String id;
	final String observerId;
	final String stationCode;
	final String formId;
	final String formHash;
	final bool proofVerified;
	final DateTime countersignAt;
	final String countersignature;

	const ObserverVerificationRecord({
		required this.id,
		required this.observerId,
		required this.stationCode,
		required this.formId,
		required this.formHash,
		required this.proofVerified,
		required this.countersignAt,
		required this.countersignature,
	});

	Map<String, dynamic> toJson() => {
		'id': id,
		'observerId': observerId,
		'stationCode': stationCode,
		'formId': formId,
		'formHash': formHash,
		'proofVerified': proofVerified,
		'countersignAt': countersignAt.toIso8601String(),
		'countersignature': countersignature,
	};
}
