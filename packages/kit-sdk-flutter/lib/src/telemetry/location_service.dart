/// Geolocator + Wi-Fi BSSID fusion; coarsens below district when accuracy > 500 m.
library;

import 'dart:async';

import 'package:geolocator/geolocator.dart';

import 'redactor.dart';
import 'signal_sink.dart';
import 'signal_types.dart';

class LocationService {
  LocationService({
    required String deviceId,
    required SignalSink sink,
    required String Function() nextTraceId,
    required String Function() nextSpanId,
    required int Function() nextCounter,
    required String Function() currentPrevHash,
    Duration activeInterval = const Duration(seconds: 1),
    Duration idleInterval = const Duration(seconds: 15),
  })  : _deviceId = deviceId,
        _sink = sink,
        _nextTraceId = nextTraceId,
        _nextSpanId = nextSpanId,
        _nextCounter = nextCounter,
        _currentPrevHash = currentPrevHash,
        _activeInterval = activeInterval,
        _idleInterval = idleInterval;

  final String _deviceId;
  final SignalSink _sink;
  final String Function() _nextTraceId;
  final String Function() _nextSpanId;
  final int Function() _nextCounter;
  final String Function() _currentPrevHash;
  final Duration _activeInterval;
  final Duration _idleInterval;

  bool _active = false;
  Timer? _timer;
  final _redactor = const Redactor();

  void setActive(bool active) {
    _active = active;
    _restart();
  }

  void start() => _restart();

  void stop() {
    _timer?.cancel();
    _timer = null;
  }

  void _restart() {
    stop();
    final interval = _active ? _activeInterval : _idleInterval;
    _timer = Timer.periodic(interval, (_) => _poll());
  }

  Future<void> _poll() async {
    try {
      final pos = await Geolocator.getCurrentPosition(
        locationSettings: const LocationSettings(accuracy: LocationAccuracy.best),
      );

      LocationSignal signal = LocationSignal(
        deviceId: _deviceId,
        occurredAt: DateTime.now().toUtc(),
        monotonicCounter: _nextCounter(),
        traceId: _nextTraceId(),
        spanId: _nextSpanId(),
        correlationId: _nextTraceId(),
        prevHash: _currentPrevHash(),
        latitudeDeg: pos.latitude,
        longitudeDeg: pos.longitude,
        accuracyM: pos.accuracy,
        altitudeM: pos.altitude,
      );

      // Coarsen if accuracy is poor.
      signal = _redactor.redact(signal) as LocationSignal;
      await _sink.accept(signal);
    } on LocationServiceDisabledException {
      // Location services off — skip silently.
    } on PermissionDeniedException {
      // Permission denied — skip silently.
    }
  }
}
