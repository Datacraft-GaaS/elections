// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
