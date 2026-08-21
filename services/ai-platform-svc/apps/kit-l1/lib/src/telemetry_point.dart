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
