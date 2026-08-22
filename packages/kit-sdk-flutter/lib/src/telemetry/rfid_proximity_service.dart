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
