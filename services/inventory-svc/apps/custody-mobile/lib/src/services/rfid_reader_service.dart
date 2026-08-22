/// BLE RFID reader interface for the custody-mobile app.
///
/// Supports Zebra RFD40 and CipherLab RS36 handhelds via BLE.
/// The service exposes a stream of [RfidScanResult] events and
/// handles pairing, connection lifecycle, and reconnection.
library rfid_reader_service;

import 'dart:async';

/// Supported BLE reader models.
enum RfidReaderModel {
  zebraRfd40,
  cipherlabRs36,
  /// Synthetic adapter used in CI / widget tests.
  synthetic,
}

/// A single RFID scan result from the reader.
class RfidScanResult {
  const RfidScanResult({
    required this.epc,
    required this.readAt,
    required this.tamperIndicator,
    required this.challengeValid,
    this.rssi,
  });

  /// 96-bit EPC as a hex string.
  final String epc;
  final DateTime readAt;

  /// True if the tag reported its tamper bit or failed to respond.
  final bool tamperIndicator;

  /// True if the HMAC challenge/response was verified by the server.
  final bool challengeValid;

  /// Signal strength in dBm, if available.
  final double? rssi;

  @override
  String toString() =>
      'RfidScanResult(epc=$epc, tamper=$tamperIndicator, challenge_valid=$challengeValid)';
}

/// Reader connection state.
enum ReaderConnectionState {
  disconnected,
  scanning,
  connecting,
  connected,
  error,
}

/// Abstract interface for a BLE RFID reader.
///
/// Concrete implementations: [ZebraRfd40ReaderService],
/// [CipherlabRs36ReaderService], [SyntheticReaderService].
abstract class RfidReaderService {
  /// Stream of scan results from the reader.
  Stream<RfidScanResult> get scanResults;

  /// Current connection state.
  ReaderConnectionState get connectionState;

  /// Stream of connection state changes.
  Stream<ReaderConnectionState> get connectionStateStream;

  /// Begin scanning for nearby BLE readers of the expected model.
  Future<void> startScan();

  /// Pair and connect to a specific device by BLE device ID.
  Future<void> connectToDevice(String deviceId);

  /// Disconnect from the current reader.
  Future<void> disconnect();

  /// Release resources.
  Future<void> dispose();
}

/// Synthetic in-memory reader for tests and CI.
class SyntheticReaderService implements RfidReaderService {
  final _scanController = StreamController<RfidScanResult>.broadcast();
  final _stateController = StreamController<ReaderConnectionState>.broadcast();
  ReaderConnectionState _state = ReaderConnectionState.disconnected;

  @override
  Stream<RfidScanResult> get scanResults => _scanController.stream;

  @override
  ReaderConnectionState get connectionState => _state;

  @override
  Stream<ReaderConnectionState> get connectionStateStream => _stateController.stream;

  @override
  Future<void> startScan() async {
    _setState(ReaderConnectionState.scanning);
  }

  @override
  Future<void> connectToDevice(String deviceId) async {
    _setState(ReaderConnectionState.connecting);
    await Future<void>.delayed(const Duration(milliseconds: 50));
    _setState(ReaderConnectionState.connected);
  }

  /// Inject a test scan result.
  void injectScan(RfidScanResult result) {
    _scanController.add(result);
  }

  /// Simulate a tamper-indicator read.
  void injectTamperRead(String epc) {
    injectScan(RfidScanResult(
      epc: epc,
      readAt: DateTime.now(),
      tamperIndicator: true,
      challengeValid: false,
    ));
  }

  @override
  Future<void> disconnect() async {
    _setState(ReaderConnectionState.disconnected);
  }

  @override
  Future<void> dispose() async {
    await _scanController.close();
    await _stateController.close();
  }

  void _setState(ReaderConnectionState s) {
    _state = s;
    _stateController.add(s);
  }
}

/// Factory that returns the correct concrete service for a given model.
RfidReaderService createReaderService(RfidReaderModel model) {
  switch (model) {
    case RfidReaderModel.synthetic:
      return SyntheticReaderService();
    case RfidReaderModel.zebraRfd40:
      // TODO: return ZebraRfd40ReaderService() once vendor SDK is integrated.
      return SyntheticReaderService();
    case RfidReaderModel.cipherlabRs36:
      // TODO: return CipherlabRs36ReaderService() once vendor SDK is integrated.
      return SyntheticReaderService();
  }
}
