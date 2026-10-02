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
