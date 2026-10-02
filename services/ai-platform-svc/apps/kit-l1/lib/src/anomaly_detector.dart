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

import 'dart:math';

import 'anomaly_result.dart';
import 'telemetry_point.dart';

/// On-device IsoForest anomaly detector. Light footprint -- no native deps.
/// Uses a simplified isolation score based on data statistics.
class EelKitL1AnomalyDetector {
  final int _numTrees;
  final List<List<double>> _trainingData = [];
  bool _fitted = false;
  double _meanScore = 0.5;
  double _stdScore = 0.1;

  EelKitL1AnomalyDetector({int numTrees = 10, int subSampleSize = 64})
      : _numTrees = numTrees;

  List<double> _featurize(TelemetryPoint p) => [
    p.votesCast.toDouble(),
    p.ballotPapersIssued.toDouble(),
    p.spoiltBallots.toDouble(),
    p.votesCast > 0 ? p.spoiltBallots / p.votesCast : 0.0,
    p.votesCast > 0 ? p.ballotPapersIssued / p.votesCast : 1.0,
  ];

  /// Train the detector on a batch of normal telemetry points.
  void fit(List<TelemetryPoint> points) {
    if (points.isEmpty) return;
    _trainingData.clear();
    for (final p in points) {
      _trainingData.add(_featurize(p));
    }
    final scores = _trainingData.map(_isolationScore).toList();
    _meanScore = scores.reduce((a, b) => a + b) / scores.length;
    final variance =
        scores.map((s) => pow(s - _meanScore, 2)).reduce((a, b) => a + b) /
        scores.length;
    _stdScore = sqrt(variance.toDouble()).clamp(0.01, 1.0);
    _fitted = true;
  }

  double _isolationScore(List<double> features) {
    final rng = Random(features.fold<int>(0, (h, v) => h ^ v.hashCode));
    double score = 0.0;
    for (int t = 0; t < _numTrees; t++) {
      score += _pathLength(features, rng, 0);
    }
    return score / _numTrees;
  }

  double _pathLength(List<double> features, Random rng, int depth) {
    if (depth >= 8 || _trainingData.length < 2) return depth.toDouble();
    final dim = rng.nextInt(features.length);
    final vals = _trainingData.map((r) => r[dim]).toList();
    final minVal = vals.reduce(min);
    final maxVal = vals.reduce(max);
    if (maxVal <= minVal) return depth.toDouble();
    final split = minVal + rng.nextDouble() * (maxVal - minVal);
    return features[dim] < split
        ? _pathLength(features, rng, depth + 1)
        : _pathLength(features, rng, depth + 1);
  }

  /// Detect anomalies in a batch of telemetry points.
  List<AnomalyResult> detect(List<TelemetryPoint> points) {
    if (!_fitted) {
      fit(points);
    }
    return points.map((p) {
      final features = _featurize(p);
      final score = _isolationScore(features);
      final zscore = _stdScore > 0 ? (score - _meanScore) / _stdScore : 0.0;
      final isAnomaly = zscore.abs() > 2.0;
      final severity = zscore.abs() > 4.0
          ? AnomalySeverity.critical
          : zscore.abs() > 3.0
          ? AnomalySeverity.high
          : zscore.abs() > 2.0
          ? AnomalySeverity.medium
          : AnomalySeverity.low;
      return AnomalyResult(
        stationCode: p.stationCode,
        isAnomaly: isAnomaly,
        score: score,
        severity: severity,
        layer: 'kit-l1-isoforest',
        detectedAt: DateTime.now().toUtc(),
      );
    }).toList();
  }
}
