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

import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

/// Iris scanner device abstraction (AntIris USB).
///
/// Mirrors the transport-driven pattern used for the RFID readers: a platform
/// channel transport talks to the native Android bridge (which wraps
/// `com.antiris.AntUSBCamera.USBControl`), and a simulated transport lets the
/// app and tests run without hardware.
///
/// Native contract (`eel/iris` MethodChannel):
///   connect()                    -> {device_id}
///   disconnect()
///   enroll({eye})                -> {eye, iso_base64}          (ISO 19794-6)
///   identify({eye, templates})   -> {user_id, score?}          (1:N against the
///                                                                 supplied gallery)
///   captureIso({eye})            -> {eye, iso_base64}
///   stop()
///   Event stream: 'connected' | 'disconnected' | 'error' | 'enrolled' |
///                 'identified' | 'captured'.

/// Which eye(s) to capture.
enum IrisEye { left, right, both }

String irisEyeName(IrisEye eye) => switch (eye) {
      IrisEye.left => 'left',
      IrisEye.right => 'right',
      IrisEye.both => 'both',
    };

IrisEye irisEyeFromName(String name) => switch (name) {
      'right' => IrisEye.right,
      'both' => IrisEye.both,
      _ => IrisEye.left,
    };

/// An enrolled or captured iris template (ISO 19794-6 bytes).
@immutable
class IrisTemplate {
  final IrisEye eye;
  final Uint8List isoBytes;

  const IrisTemplate({required this.eye, required this.isoBytes});
}

/// A 1:N identification result.
@immutable
class IrisMatch {
  final String userId;
  final double? score;

  const IrisMatch({required this.userId, this.score});
}

/// Base event emitted by an [IrisScannerTransport].
@immutable
sealed class IrisEvent {
  const IrisEvent();
}

class IrisConnected extends IrisEvent {
  final String deviceId;
  const IrisConnected(this.deviceId);
}

class IrisDisconnected extends IrisEvent {
  const IrisDisconnected();
}

class IrisEnrolled extends IrisEvent {
  final IrisTemplate template;
  const IrisEnrolled(this.template);
}

class IrisIdentified extends IrisEvent {
  final IrisMatch match;
  const IrisIdentified(this.match);
}

class IrisCaptured extends IrisEvent {
  final IrisTemplate template;
  const IrisCaptured(this.template);
}

class IrisError extends IrisEvent {
  final String message;
  const IrisError(this.message);
}

/// Raw transport for an iris scanner.
abstract class IrisScannerTransport {
  Stream<IrisEvent> get events;
  Future<void> connect();
  Future<void> disconnect();
  Future<void> enroll({required IrisEye eye});
  Future<void> identify({
    required IrisEye eye,
    List<IrisTemplate> templates = const [],
  });
  Future<void> captureIso({required IrisEye eye});
  Future<void> stop();
  Future<void> dispose();
}

/// Platform-channel transport speaking to the native AntIris bridge.
class PlatformChannelIrisTransport implements IrisScannerTransport {
  PlatformChannelIrisTransport({String channel = 'eel/iris'})
      : _channel = MethodChannel(channel) {
    _channel.setMethodCallHandler(_onMethodCall);
  }

  final MethodChannel _channel;
  final _controller = StreamController<IrisEvent>.broadcast();

  @override
  Stream<IrisEvent> get events => _controller.stream;

  Future<dynamic> _onMethodCall(MethodCall call) async {
    switch (call.method) {
      case 'connected':
        _controller.add(IrisConnected(call.arguments as String));
      case 'disconnected':
        _controller.add(const IrisDisconnected());
      case 'error':
        _controller.add(IrisError(call.arguments as String? ?? 'iris error'));
      case 'enrolled':
        final args = call.arguments as Map;
        _controller.add(IrisEnrolled(_templateFromMap(args)));
      case 'identified':
        final args = call.arguments as Map;
        _controller.add(IrisIdentified(IrisMatch(
          userId: args['user_id'] as String? ?? '',
          score: (args['score'] as num?)?.toDouble(),
        )));
      case 'captured':
        _controller.add(IrisCaptured(_templateFromMap(call.arguments as Map)));
    }
  }

  @override
  Future<void> connect() async {
    await _channel.invokeMethod('connect');
  }

  @override
  Future<void> disconnect() async {
    await _channel.invokeMethod('disconnect');
  }

  @override
  Future<void> enroll({required IrisEye eye}) async {
    await _channel.invokeMethod('enroll', {'eye': irisEyeName(eye)});
  }

  @override
  Future<void> identify({
    required IrisEye eye,
    List<IrisTemplate> templates = const [],
  }) async {
    await _channel.invokeMethod('identify', {
      'eye': irisEyeName(eye),
      'templates': templates.map((t) => base64Encode(t.isoBytes)).toList(),
    });
  }

  @override
  Future<void> captureIso({required IrisEye eye}) async {
    await _channel.invokeMethod('captureIso', {'eye': irisEyeName(eye)});
  }

  @override
  Future<void> stop() async {
    await _channel.invokeMethod('stop');
  }

  @override
  Future<void> dispose() async {
    _channel.setMethodCallHandler(null);
    await _controller.close();
  }
}

IrisTemplate _templateFromMap(Map args) => IrisTemplate(
      eye: irisEyeFromName(args['eye'] as String? ?? 'left'),
      isoBytes: base64Decode(args['iso_base64'] as String),
    );

/// Simulated transport for tests and demos; produces a deterministic template
/// and never touches a platform channel.
class SimulatedIrisTransport implements IrisScannerTransport {
  SimulatedIrisTransport();

  final _controller = StreamController<IrisEvent>.broadcast();

  @override
  Stream<IrisEvent> get events => _controller.stream;

  @override
  Future<void> connect() async {
    _controller.add(const IrisConnected('SIM-IRIS-01'));
  }

  @override
  Future<void> disconnect() async {
    _controller.add(const IrisDisconnected());
  }

  @override
  Future<void> enroll({required IrisEye eye}) async {
    await Future<void>.delayed(const Duration(milliseconds: 50));
    _controller.add(IrisEnrolled(_fakeTemplate(eye)));
  }

  @override
  Future<void> identify({
    required IrisEye eye,
    List<IrisTemplate> templates = const [],
  }) async {
    await Future<void>.delayed(const Duration(milliseconds: 50));
    _controller.add(const IrisIdentified(IrisMatch(userId: 'VOTER-0001', score: 0.98)));
  }

  @override
  Future<void> captureIso({required IrisEye eye}) async {
    await Future<void>.delayed(const Duration(milliseconds: 50));
    _controller.add(IrisCaptured(_fakeTemplate(eye)));
  }

  @override
  Future<void> stop() async {}

  @override
  Future<void> dispose() async {
    await _controller.close();
  }

  IrisTemplate _fakeTemplate(IrisEye eye) => IrisTemplate(
        eye: eye,
        isoBytes: Uint8List.fromList(List.filled(32, eye.index + 1)),
      );
}

/// Stateful scanner facade over a transport. Subscribes to transport events
/// and exposes the latest template/match plus connection state.
class IrisScanner extends ChangeNotifier {
  IrisScanner({IrisScannerTransport? transport})
      : _transport = transport ?? PlatformChannelIrisTransport() {
    _sub = _transport.events.listen(_onEvent);
  }

  final IrisScannerTransport _transport;
  late final StreamSubscription<IrisEvent> _sub;

  bool _connected = false;
  bool _busy = false;
  String? _error;
  IrisTemplate? _lastTemplate;
  IrisMatch? _lastMatch;

  bool get connected => _connected;
  bool get busy => _busy;
  String? get error => _error;
  IrisTemplate? get lastTemplate => _lastTemplate;
  IrisMatch? get lastMatch => _lastMatch;

  void _onEvent(IrisEvent event) {
    switch (event) {
      case IrisConnected():
        _connected = true;
        _busy = false;
        _error = null;
      case IrisDisconnected():
        _connected = false;
      case IrisEnrolled(:final template):
        _lastTemplate = template;
        _busy = false;
      case IrisIdentified(:final match):
        _lastMatch = match;
        _busy = false;
      case IrisCaptured(:final template):
        _lastTemplate = template;
        _busy = false;
      case IrisError(:final message):
        _error = message;
        _busy = false;
    }
    notifyListeners();
  }

  Future<void> connect() async {
    _busy = true;
    notifyListeners();
    await _transport.connect();
  }

  Future<void> disconnect() => _transport.disconnect();

  Future<void> enroll(IrisEye eye) async {
    _busy = true;
    notifyListeners();
    await _transport.enroll(eye: eye);
  }

  Future<void> identify(
    IrisEye eye, {
    List<IrisTemplate> templates = const [],
  }) async {
    _busy = true;
    notifyListeners();
    await _transport.identify(eye: eye, templates: templates);
  }

  Future<void> captureIso(IrisEye eye) async {
    _busy = true;
    notifyListeners();
    await _transport.captureIso(eye: eye);
  }

  Future<void> stop() => _transport.stop();

  @override
  void dispose() {
    _sub.cancel();
    _transport.dispose();
    super.dispose();
  }
}
