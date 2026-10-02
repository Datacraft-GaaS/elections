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
