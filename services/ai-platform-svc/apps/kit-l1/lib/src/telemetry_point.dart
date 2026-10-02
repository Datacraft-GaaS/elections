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

class TelemetryPoint {
  final String stationCode;
  final DateTime timestamp;
  final int votesCast;
  final int ballotPapersIssued;
  final int spoiltBallots;
  final String kitId;

  const TelemetryPoint({
    required this.stationCode,
    required this.timestamp,
    required this.votesCast,
    required this.ballotPapersIssued,
    required this.spoiltBallots,
    required this.kitId,
  });

  Map<String, dynamic> toJson() => {
    'station_code': stationCode,
    'timestamp': timestamp.toIso8601String(),
    'votes_cast': votesCast,
    'ballot_papers_issued': ballotPapersIssued,
    'spoilt_ballots': spoiltBallots,
    'kit_id': kitId,
  };
}
