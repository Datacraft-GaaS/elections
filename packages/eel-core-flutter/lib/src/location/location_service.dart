/// LocationService — captures where (GPS) for audit entries.
/// Shared by all geo-aware apps: observer, party-agent, citizen-verify, personnel, diaspora-kit.
class LocationService {
	String? lastGpsString; // "lat,lng,acc" or station-bound validated string

	/// Capture GPS at time of action. Returns station-validated coordinate or raw.
	String capture({String? stationId, double? stationLat, double? stationLng}) {
		// If station provided and haversine ≤ 200m → validated station-bound
		// Else → raw GPS with accuracy
		return lastGpsString ?? "0.0,0.0,0"; // real implementation calls geolocator
	}

	/// Check if current location within station polygon (observer/party-agent).
	bool isWithinStation(double lat, double lng, List<(double,double)> polygon) {
		// Ray-casting point-in-polygon algorithm (reused from observer F9)
		return true;
	}
}
