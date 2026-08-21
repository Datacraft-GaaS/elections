enum AnomalySeverity { low, medium, high, critical }

class AnomalyResult {
  final String stationCode;
  final bool isAnomaly;
  final double score;
  final AnomalySeverity severity;
  final String layer;
  final DateTime detectedAt;

  const AnomalyResult({
    required this.stationCode,
    required this.isAnomaly,
    required this.score,
    required this.severity,
    required this.layer,
    required this.detectedAt,
  });
}
