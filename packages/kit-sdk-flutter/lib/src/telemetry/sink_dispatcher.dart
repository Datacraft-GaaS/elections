/// Batch dispatcher: drains [TelemetryOfflineQueue] and POST-s to ingest.
///
/// Endpoint: POST /v1/telemetry/ingest on field-mobile-svc.
/// Protocol: HTTPS + SVID header + Idempotency-Key per batch.
/// Batch size: up to 1000 signals per request.
/// Retry: exponential backoff; backpressure via [_maxInFlight].
library;

import 'dart:async';
import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:uuid/uuid.dart';

import 'offline_queue.dart';

const _uuid = Uuid();

class SinkDispatcher {
  SinkDispatcher({
    required TelemetryOfflineQueue queue,
    required Dio http,
    required String ingestBaseUrl,
    required String deviceSvid,
    int batchSize = 1000,
    int maxInFlight = 3,
  })  : _queue = queue,
        _http = http,
        _ingestBaseUrl = ingestBaseUrl,
        _svid = deviceSvid,
        _batchSize = batchSize,
        _maxInFlight = maxInFlight;

  final TelemetryOfflineQueue _queue;
  final Dio _http;
  final String _ingestBaseUrl;
  final String _svid;
  final int _batchSize;
  final int _maxInFlight;

  int _inFlight = 0;
  bool _running = false;
  Timer? _timer;

  /// Start periodic dispatch every [intervalMs] milliseconds.
  void start({int intervalMs = 5000}) {
    if (_running) return;
    _running = true;
    _timer = Timer.periodic(Duration(milliseconds: intervalMs), (_) => _dispatch());
  }

  void stop() {
    _running = false;
    _timer?.cancel();
    _timer = null;
  }

  Future<void> _dispatch() async {
    if (_inFlight >= _maxInFlight) return;
    final batch = _queue.drainBatch(maxSize: _batchSize);
    if (batch.isEmpty) return;
    _inFlight++;
    try {
      final idempotencyKey = _uuid.v4();
      final response = await _http.post(
        '$_ingestBaseUrl/v1/telemetry/ingest',
        data: jsonEncode({'signals': batch.map((e) => e.signalJson).toList()}),
        options: Options(
          headers: {
            'Content-Type': 'application/json',
            'X-SVID': _svid,
            'Idempotency-Key': idempotencyKey,
          },
          sendTimeout: const Duration(seconds: 30),
          receiveTimeout: const Duration(seconds: 30),
        ),
      );
      if (response.statusCode == 200 || response.statusCode == 202) {
        await _queue.acknowledge(batch.map((e) => e.id).toList());
      } else {
        await _queue.scheduleRetry(batch.map((e) => e.id).toList());
      }
    } on DioException {
      await _queue.scheduleRetry(batch.map((e) => e.id).toList());
    } finally {
      _inFlight--;
    }
  }

  /// Trigger an immediate dispatch attempt (e.g. on reconnect).
  Future<void> triggerNow() => _dispatch();
}
