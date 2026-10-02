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
