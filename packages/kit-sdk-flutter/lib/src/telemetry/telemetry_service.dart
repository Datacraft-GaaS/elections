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

/// Top-level TelemetryService — wires all sub-services and exposes the public API.
///
/// Usage in each app's main.dart:
///   await TelemetryService.init(appId: 'bvr-kit', personaRole: 'enrolment_officer');
library;

import 'dart:async';
import 'dart:convert';

import 'package:crypto/crypto.dart';
import 'package:dio/dio.dart';
import 'package:uuid/uuid.dart';

import 'adaptive_sampling.dart';
import 'activity_tracker.dart';
import 'attestation_pulse.dart';
import 'battery_monitor.dart';
import 'hash_chain.dart';
import 'location_service.dart';
import 'network_watcher.dart';
import 'offline_queue.dart';
import 'redactor.dart';
import 'sensors_service.dart';
import 'session_recorder.dart';
import 'signal_sink.dart';
import 'signal_types.dart';
import 'signer.dart';
import 'sink_dispatcher.dart';

const _uuid = Uuid();

/// Ingest endpoint base URL; override per environment.
const String _defaultIngestBase = 'https://field-mobile.eel.internal';

/// Singleton telemetry coordinator.
class TelemetryService {
  TelemetryService._({
    required this.appId,
    required this.personaRole,
    required TelemetryOfflineQueue queue,
    required HashChain chain,
    required Signer signer,
    required AdaptiveSampling sampling,
  })  : _queue = queue,
        _chain = chain,
        _signer = signer,
        _sampling = sampling;

  final String appId;
  final String personaRole;

  final TelemetryOfflineQueue _queue;
  final HashChain _chain;
  final Signer _signer;
  final AdaptiveSampling _sampling;

  int _counter = 0;
  int _batteryPercent = 100;
  bool _workflowActive = false;

  late SensorsService _sensors;
  late LocationService _location;
  late BatteryMonitor _battery;
  late NetworkWatcher _network;
  late AttestationPulse _attestation;
  late SinkDispatcher _dispatcher;
  late SessionRecorder _sessionRecorder;

  // Public navigator observer — attach to MaterialApp.navigatorObservers.
  late TelemetryNavigatorObserver navigatorObserver;

  static TelemetryService? _instance;

  static TelemetryService get instance {
    assert(_instance != null, 'TelemetryService.init() must be called first');
    return _instance!;
  }

  /// Initialise and start all telemetry sub-services.
  ///
  /// Call once inside `main()` before `runApp()`.
  static Future<TelemetryService> init({
    required String appId,
    required String personaRole,
    String? deviceId,
    String? ingestBaseUrl,
    String? deviceKey,
    TelemetryQueueStorage? storage,
    Future<Map<String, Object?>> Function()? attestProvider,
  }) async {
    if (_instance != null) return _instance!;

    final resolvedDeviceId = deviceId ?? _uuid.v4();
    final resolvedKey = deviceKey ?? 'dev-key-$resolvedDeviceId';
    final resolvedStorage = storage ?? InMemoryTelemetryStorage();

    final chain = HashChain();
    final signerImpl = HmacSigner(deviceKey: resolvedKey);
    const redactor = Redactor();
    const sampling = AdaptiveSampling();

    // Signing sink wraps the raw queue: hashes then signs each signal.
    final queue = TelemetryOfflineQueue(storage: resolvedStorage);
    await queue.init();

    final signingSink = _SigningAndChainSink(
      inner: queue,
      chain: chain,
      signer: signerImpl,
      redactor: redactor,
    );

    final svc = TelemetryService._(
      appId: appId,
      personaRole: personaRole,
      queue: queue,
      chain: chain,
      signer: signerImpl,
      sampling: sampling,
    );

    String traceId() => const Uuid().v4();
    String spanId() => const Uuid().v4();
    int counter() => svc._counter++;
    String prevHash() => chain.tail;

    svc._sensors = SensorsService(
      deviceId: resolvedDeviceId,
      sink: signingSink,
      nextTraceId: traceId,
      nextSpanId: spanId,
      nextCounter: counter,
      currentPrevHash: prevHash,
      sampling: sampling,
    );

    svc._location = LocationService(
      deviceId: resolvedDeviceId,
      sink: signingSink,
      nextTraceId: traceId,
      nextSpanId: spanId,
      nextCounter: counter,
      currentPrevHash: prevHash,
    );

    svc._battery = BatteryMonitor(
      deviceId: resolvedDeviceId,
      sink: signingSink,
      nextTraceId: traceId,
      nextSpanId: spanId,
      nextCounter: counter,
      currentPrevHash: prevHash,
      onBatteryUpdate: (pct) {
        svc._batteryPercent = pct;
        svc._updateSamplingMode();
      },
    );

    svc._network = NetworkWatcher(
      deviceId: resolvedDeviceId,
      sink: signingSink,
      nextTraceId: traceId,
      nextSpanId: spanId,
      nextCounter: counter,
      currentPrevHash: prevHash,
    );

    svc._attestation = AttestationPulse(
      deviceId: resolvedDeviceId,
      sink: signingSink,
      nextTraceId: traceId,
      nextSpanId: spanId,
      nextCounter: counter,
      currentPrevHash: prevHash,
      attestProvider: attestProvider ?? () async => {'status': 'unknown', 'kitCertValid': false},
    );

    svc._sessionRecorder = SessionRecorder(
      deviceId: resolvedDeviceId,
      sink: signingSink,
      nextTraceId: traceId,
      nextSpanId: spanId,
      nextCounter: counter,
      currentPrevHash: prevHash,
    );

    svc.navigatorObserver = TelemetryNavigatorObserver(
      deviceId: resolvedDeviceId,
      sink: signingSink,
      nextTraceId: traceId,
      nextSpanId: spanId,
      nextCounter: counter,
      currentPrevHash: prevHash,
    );

    final dio = Dio();
    svc._dispatcher = SinkDispatcher(
      queue: queue,
      http: dio,
      ingestBaseUrl: ingestBaseUrl ?? _defaultIngestBase,
      deviceSvid: resolvedDeviceId,
    );

    // Start all sub-services.
    svc._sensors.start();
    svc._location.start();
    svc._battery.start();
    svc._network.start();
    svc._attestation.start();
    svc._dispatcher.start();

    _instance = svc;
    return svc;
  }

  void _updateSamplingMode() {
    final mode = _sampling.mode(
      batteryPercent: _batteryPercent,
      workflowActive: _workflowActive,
    );
    _sensors.updateMode(mode);
    _location.setActive(_workflowActive);
  }

  // ---------------------------------------------------------------------------
  // Public API
  // ---------------------------------------------------------------------------

  void markWorkflowStep(String name) {
    _workflowActive = true;
    _updateSamplingMode();
    navigatorObserver.markWorkflowStep(name);
  }

  Future<void> markLogin(String actorSvid) =>
      _sessionRecorder.recordLogin(actorSvid);

  Future<void> markLogout(String actorSvid) =>
      _sessionRecorder.recordLogout(actorSvid);

  /// Emit a [ReadSignal] for [entityId] — the ID is hashed before egress.
  Future<void> markRead(String entityId, {String entityKind = 'voter_record'}) async {
    // Hash the raw ID before it enters the queue.
    final idHash = sha256.convert(utf8.encode(entityId)).toString();
    final signal = ReadSignal(
      deviceId: 'device',
      occurredAt: DateTime.now().toUtc(),
      monotonicCounter: _counter++,
      traceId: _uuid.v4(),
      spanId: _uuid.v4(),
      correlationId: _uuid.v4(),
      prevHash: _chain.tail,
      entityKind: entityKind,
      entityIdHash: idHash,
    );
    _chain.attach(signal);
    await _signer.sign(signal);
    await _queue.accept(signal);
  }

  /// Emit a [WriteSignal] for [entityId] — the ID is hashed before egress.
  Future<void> markWrite(String entityId, {String entityKind = 'enrolment'}) async {
    final idHash = sha256.convert(utf8.encode(entityId)).toString();
    final signal = WriteSignal(
      deviceId: 'device',
      occurredAt: DateTime.now().toUtc(),
      monotonicCounter: _counter++,
      traceId: _uuid.v4(),
      spanId: _uuid.v4(),
      correlationId: _uuid.v4(),
      prevHash: _chain.tail,
      entityKind: entityKind,
      entityIdHash: idHash,
    );
    _chain.attach(signal);
    await _signer.sign(signal);
    await _queue.accept(signal);
  }

  /// Reset singleton — used in tests only.
  static void resetForTest() => _instance = null;
}

/// Internal sink that applies hash-chain + signing + redaction before
/// forwarding to the [TelemetryOfflineQueue].
class _SigningAndChainSink implements SignalSink {
  _SigningAndChainSink({
    required SignalSink inner,
    required HashChain chain,
    required Signer signer,
    required Redactor redactor,
  })  : _inner = inner,
        _chain = chain,
        _signer = signer,
        _redactor = redactor;

  final SignalSink _inner;
  final HashChain _chain;
  final Signer _signer;
  final Redactor _redactor;

  @override
  Future<void> accept(Signal signal) async {
    final redacted = _redactor.redact(signal);
    _chain.attach(redacted);
    await _signer.sign(redacted);
    await _inner.accept(redacted);
  }

  @override
  Future<void> flush() => _inner.flush();

  @override
  Future<void> close() => _inner.close();
}
