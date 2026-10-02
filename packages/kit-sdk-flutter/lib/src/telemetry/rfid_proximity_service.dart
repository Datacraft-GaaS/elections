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

/// BLE scan adapter for RFID proximity (ballot-box proximity detection).
///
/// Integrates with the existing rfid_reader_service.dart in custody-mobile
/// via a pluggable [RfidScanCallback] — the host app wires its own reader.
library;

import 'signal_sink.dart';
import 'signal_types.dart';

/// Callback invoked by the host's RFID/BLE scan layer with each detected tag.
typedef RfidScanCallback = void Function(RfidTag tag);

/// A detected RFID/BLE tag.
class RfidTag {
  const RfidTag({
    required this.epc,
    required this.rssi,
    this.sealTagStatus,
    this.boxBodyId,
  });

  final String epc;
  final int rssi;
  final String? sealTagStatus;
  final String? boxBodyId;
}

/// Service that converts RFID scan events into [ProximalRfidSignal]s.
class RfidProximityService {
  RfidProximityService({
    required String deviceId,
    required SignalSink sink,
    required String Function() nextTraceId,
    required String Function() nextSpanId,
    required int Function() nextCounter,
    required String Function() currentPrevHash,
  })  : _deviceId = deviceId,
        _sink = sink,
        _nextTraceId = nextTraceId,
        _nextSpanId = nextSpanId,
        _nextCounter = nextCounter,
        _currentPrevHash = currentPrevHash;

  final String _deviceId;
  final SignalSink _sink;
  final String Function() _nextTraceId;
  final String Function() _nextSpanId;
  final int Function() _nextCounter;
  final String Function() _currentPrevHash;

  /// Wire this callback to the host app's BLE/RFID scan output.
  late final RfidScanCallback onTag = _onTag;

  void _onTag(RfidTag tag) {
    final signal = ProximalRfidSignal(
      deviceId: _deviceId,
      occurredAt: DateTime.now().toUtc(),
      monotonicCounter: _nextCounter(),
      traceId: _nextTraceId(),
      spanId: _nextSpanId(),
      correlationId: _nextTraceId(),
      prevHash: _currentPrevHash(),
      rfidEpc: tag.epc,
      rssi: tag.rssi,
      sealTagStatus: tag.sealTagStatus,
      boxBodyId: tag.boxBodyId,
    );
    _sink.accept(signal);
  }
}
