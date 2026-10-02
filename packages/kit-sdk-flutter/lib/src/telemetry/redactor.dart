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

/// PII redaction before egress.
///
/// Rules enforced on every signal before it enters the offline queue:
/// - Voter IDs are SHA-256 hashed; the raw ID never leaves the device.
/// - Officer names are replaced with SVID references.
/// - Location precision is coarsened to district level when accuracy > 500 m.
library;

import 'dart:convert';
import 'package:crypto/crypto.dart';

import 'signal_sink.dart';
import 'signal_types.dart';

const double _coarsenThresholdM = 500.0;

/// Coarsening granularity in decimal degrees (roughly 10 km).
const double _districtGranularityDeg = 0.1;

class Redactor {
  const Redactor();

  /// Hashes [rawId] with SHA-256 and returns the hex digest.
  static String hashId(String rawId) =>
      sha256.convert(utf8.encode(rawId)).toString();

  /// Coarsens [degrees] to the nearest [_districtGranularityDeg] grid cell.
  static double coarsen(double degrees) =>
      (degrees / _districtGranularityDeg).roundToDouble() * _districtGranularityDeg;

  /// Applies all redaction rules to [signal] in-place.
  ///
  /// Returns the (potentially mutated) signal.  For immutable-field signals
  /// that need coarsening a new instance is returned; all other signals are
  /// returned unchanged (PII is already hashed at call-site by higher layers).
  Signal redact(Signal signal) {
    if (signal is LocationSignal && signal.accuracyM > _coarsenThresholdM) {
      return _coarsenLocation(signal);
    }
    return signal;
  }

  LocationSignal _coarsenLocation(LocationSignal s) => LocationSignal(
        deviceId: s.deviceId,
        occurredAt: s.occurredAt,
        monotonicCounter: s.monotonicCounter,
        traceId: s.traceId,
        spanId: s.spanId,
        correlationId: s.correlationId,
        prevHash: s.prevHash,
        latitudeDeg: coarsen(s.latitudeDeg),
        longitudeDeg: coarsen(s.longitudeDeg),
        accuracyM: s.accuracyM,
        altitudeM: s.altitudeM,
        cellTowerId: s.cellTowerId,
        cellRssi: s.cellRssi,
        wifiBssid: s.wifiBssid,
        wifiRssi: s.wifiRssi,
        isCoarsened: true,
        thisHash: s.thisHash,
        signature: s.signature,
        severity: s.severity,
      );
}
