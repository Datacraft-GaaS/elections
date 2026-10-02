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
