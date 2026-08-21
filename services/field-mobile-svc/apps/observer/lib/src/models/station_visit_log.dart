class StationVisitLog {
	final String id;
	final String observerId;
	final String stationCode;
	final DateTime checkInAt;
	final DateTime? checkOutAt;
	final double gpsLat;
	final double gpsLon;
	final String observationNote;
	final String signature;

	const StationVisitLog({
		required this.id,
		required this.observerId,
		required this.stationCode,
		required this.checkInAt,
		this.checkOutAt,
		required this.gpsLat,
		required this.gpsLon,
		required this.observationNote,
		required this.signature,
	});

	Map<String, dynamic> toJson() => {
		'id': id,
		'observerId': observerId,
		'stationCode': stationCode,
		'checkInAt': checkInAt.toIso8601String(),
		'checkOutAt': checkOutAt?.toIso8601String(),
		'gpsLat': gpsLat,
		'gpsLon': gpsLon,
		'observationNote': observationNote,
		'signature': signature,
	};
}
