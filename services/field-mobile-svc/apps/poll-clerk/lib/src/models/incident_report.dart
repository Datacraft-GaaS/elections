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
