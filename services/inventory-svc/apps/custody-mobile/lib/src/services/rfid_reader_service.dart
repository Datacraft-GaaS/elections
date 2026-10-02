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

/// BLE RFID reader interface for the custody-mobile app.
///
/// Supports Zebra RFD40 and CipherLab RS36 handhelds via BLE.
/// The service exposes a stream of [RfidScanResult] events and
/// handles pairing, connection lifecycle, and reconnection.
library rfid_reader_service;

import 'dart:async';

import 'package:flutter/services.dart';

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

// ---------------------------------------------------------------------------
// Transport abstraction
// ---------------------------------------------------------------------------

/// Raw event emitted by a reader transport.
sealed class RfidTransportEvent {
  const RfidTransportEvent();
}

/// A tag read reported by the reader hardware.
class RfidTagReadEvent extends RfidTransportEvent {
  const RfidTagReadEvent({
    required this.epc,
    required this.tamperBit,
    required this.challengeVerified,
    this.rssi,
  });

  /// 96-bit EPC as a hex string.
  final String epc;

  /// Raw tamper bit from the tag memory bank.
  final bool tamperBit;

  /// True when the HMAC challenge/response for this tag was verified.
  final bool challengeVerified;

  /// Signal strength in dBm, if the radio reports it.
  final double? rssi;
}

/// Transport established a pairing with a reader device.
class RfidConnectedEvent extends RfidTransportEvent {
  const RfidConnectedEvent(this.deviceId);

  final String deviceId;
}

/// Transport dropped the reader connection.
class RfidDisconnectedEvent extends RfidTransportEvent {
  const RfidDisconnectedEvent();
}

/// Transport failed; [message] describes the failure.
class RfidTransportErrorEvent extends RfidTransportEvent {
  const RfidTransportErrorEvent(this.message);

  final String message;
}

/// Pluggable link between an [RfidReaderService] and reader hardware.
///
/// Production implementations bridge to vendor SDKs over platform channels;
/// tests inject in-memory fakes.
abstract class RfidReaderTransport {
  /// Stream of raw transport events.
  Stream<RfidTransportEvent> get events;

  /// Begin BLE discovery of nearby readers of this vendor's models.
  Future<void> discover();

  /// Pair and connect to a specific device. Tag reads start flowing on
  /// [events] once connected.
  Future<void> connect(String deviceId);

  /// Drop the connection to the current reader.
  Future<void> disconnect();

  /// Release transport resources.
  Future<void> dispose();
}

/// Platform-channel transport speaking to a native vendor SDK bridge.
///
/// Control operations use [MethodChannel]; events arrive on [EventChannel].
/// The native side is registered by the Android host app. When no handler is
/// registered every call throws, which the owning service maps to
/// [ReaderConnectionState.error].
class PlatformChannelReaderTransport implements RfidReaderTransport {
  PlatformChannelReaderTransport(String vendor)
      : _control = MethodChannel('eel.custody.rfid/$vendor/control'),
        _events = EventChannel('eel.custody.rfid/$vendor/events');

  final MethodChannel _control;
  final EventChannel _events;
  Stream<RfidTransportEvent>? _eventStream;

  @override
  Stream<RfidTransportEvent> get events {
    return _eventStream ??= _events.receiveBroadcastStream().expand(
      (dynamic raw) {
        final event = _decodeEvent(raw);
        return event == null ? const <RfidTransportEvent>[] : [event];
      },
    );
  }

  static RfidTransportEvent? _decodeEvent(dynamic raw) {
    if (raw is! Map) return null;
    switch (raw['type']) {
      case 'tagRead':
        return RfidTagReadEvent(
          epc: raw['epc'] as String? ?? '',
          tamperBit: raw['tamperBit'] == true,
          challengeVerified: raw['challengeVerified'] == true,
          rssi: (raw['rssi'] as num?)?.toDouble(),
        );
      case 'connected':
        return RfidConnectedEvent(raw['deviceId'] as String? ?? '');
      case 'disconnected':
        return const RfidDisconnectedEvent();
      case 'error':
        return RfidTransportErrorEvent(
            raw['message'] as String? ?? 'unknown transport error');
      default:
        return null;
    }
  }

  @override
  Future<void> discover() => _control.invokeMethod<void>('discover');

  @override
  Future<void> connect(String deviceId) =>
      _control.invokeMethod<void>('connect', {'deviceId': deviceId});

  @override
  Future<void> disconnect() => _control.invokeMethod<void>('disconnect');

  @override
  Future<void> dispose() async {}
}

// ---------------------------------------------------------------------------
// Transport-driven reader services
// ---------------------------------------------------------------------------

/// Shared connection lifecycle for transport-backed reader services.
///
/// Subclasses supply the vendor-specific [transport]; state transitions and
/// event mapping are handled here.
abstract class TransportDrivenReaderService implements RfidReaderService {
  TransportDrivenReaderService(this.transport);

  /// Link to reader hardware.
  final RfidReaderTransport transport;

  final _scanController = StreamController<RfidScanResult>.broadcast();
  final _stateController = StreamController<ReaderConnectionState>.broadcast();
  StreamSubscription<RfidTransportEvent>? _eventSub;
  ReaderConnectionState _state = ReaderConnectionState.disconnected;
  bool _disposed = false;

  @override
  Stream<RfidScanResult> get scanResults => _scanController.stream;

  @override
  ReaderConnectionState get connectionState => _state;

  @override
  Stream<ReaderConnectionState> get connectionStateStream =>
      _stateController.stream;

  @override
  Future<void> startScan() async {
    _ensureNotDisposed();
    _setState(ReaderConnectionState.scanning);
    try {
      await transport.discover();
    } catch (error) {
      _fail(error);
    }
  }

  @override
  Future<void> connectToDevice(String deviceId) async {
    _ensureNotDisposed();
    _listenEvents();
    _setState(ReaderConnectionState.connecting);
    try {
      await transport.connect(deviceId);
    } catch (error) {
      _fail(error);
    }
  }

  void _listenEvents() {
    if (_eventSub != null) return;
    _eventSub = transport.events.listen(
      _onTransportEvent,
      onError: (Object error) => _fail(error),
    );
  }

  void _onTransportEvent(RfidTransportEvent event) {
    if (_disposed) return;
    switch (event) {
      case RfidTagReadEvent():
        if (event.epc.isEmpty) return;
        _scanController.add(RfidScanResult(
          epc: event.epc.toUpperCase(),
          readAt: DateTime.now(),
          tamperIndicator: event.tamperBit,
          challengeValid: event.challengeVerified,
          rssi: event.rssi,
        ));
      case RfidConnectedEvent():
        _setState(ReaderConnectionState.connected);
      case RfidDisconnectedEvent():
        _setState(ReaderConnectionState.disconnected);
      case RfidTransportErrorEvent():
        _setState(ReaderConnectionState.error);
    }
  }

  @override
  Future<void> disconnect() async {
    if (_disposed) return;
    try {
      await transport.disconnect();
    } catch (error) {
      _fail(error);
      return;
    }
    _setState(ReaderConnectionState.disconnected);
  }

  @override
  Future<void> dispose() async {
    if (_disposed) return;
    _disposed = true;
    await _eventSub?.cancel();
    _eventSub = null;
    await transport.dispose();
    await _scanController.close();
    await _stateController.close();
  }

  void _fail(Object error) {
    _setState(ReaderConnectionState.error);
  }

  void _ensureNotDisposed() {
    if (_disposed) throw StateError('$runtimeType has been disposed');
  }

  void _setState(ReaderConnectionState s) {
    if (_state == s) return;
    _state = s;
    _stateController.add(s);
  }
}

/// Zebra RFD40 reader service backed by the native Zebra SDK bridge.
class ZebraRfd40ReaderService extends TransportDrivenReaderService {
  ZebraRfd40ReaderService({RfidReaderTransport? transport})
      : super(transport ?? PlatformChannelReaderTransport('zebra_rfd40'));
}

/// CipherLab RS36 reader service backed by the native CipherLab SDK bridge.
class CipherlabRs36ReaderService extends TransportDrivenReaderService {
  CipherlabRs36ReaderService({RfidReaderTransport? transport})
      : super(transport ?? PlatformChannelReaderTransport('cipherlab_rs36'));
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

/// Factory that returns the correct concrete service for a given model.
///
/// Vendor services talk to their native SDK bridge over platform channels;
/// pass an explicit [transport] when constructing them directly for tests or
/// alternative links.
RfidReaderService createReaderService(RfidReaderModel model) {
  switch (model) {
    case RfidReaderModel.synthetic:
      return SyntheticReaderService();
    case RfidReaderModel.zebraRfd40:
      return ZebraRfd40ReaderService();
    case RfidReaderModel.cipherlabRs36:
      return CipherlabRs36ReaderService();
  }
}
