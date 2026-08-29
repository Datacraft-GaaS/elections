import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

/// FAP60 fingerprint scanner abstraction (ZAZ / FingerUp FAP60).
///
/// Vendor driver: sdks/2.fap60-android (FUFingerSDK + FPAPI + FingerAlgo).
/// Native libs: libUVCCamera, libFingerILA, libFpStdAPI, libZAZ_FpStdAPI, etc.
/// for armeabi-v7a / arm64-v8a.
///
/// Native contract (`eel/fingerprint` MethodChannel):
///   connect()                                    -> {device_id}
///   disconnect()
///   captureFlat()                                -> {raw_base64,width,height,quality,wsq_base64}  (300x400 single finger)
///   captureSlap({hand})                          -> {fingers:[{raw_base64,wsq_base64,quality,position}]} (1600x1500 split 4)
///   captureRoll()                                -> {raw_base64,width,height,quality}             (800x750 roll)
///   enroll({count})                              -> {templates:[{iso_base64},...]}                (ISO 19794-2 via FPAPI)
///   identify({candidates})                       -> {matched_id, score}                           (1:N via FPAPI CompareTemplates)
///   setDryWet({level})  // 3 wet, 4 normal, 5 dry
///   beep({type})
///   Event stream: 'connected' | 'disconnected' | 'error' | 'captured' |
///                 'enrolled' | 'identified'

enum FingerPosition {
  leftThumb,
  leftIndex,
  leftMiddle,
  leftRing,
  leftLittle,
  rightThumb,
  rightIndex,
  rightMiddle,
  rightRing,
  rightLittle,
}

String fingerPositionName(FingerPosition p) => switch (p) {
      FingerPosition.leftThumb => 'left_thumb',
      FingerPosition.leftIndex => 'left_index',
      FingerPosition.leftMiddle => 'left_middle',
      FingerPosition.leftRing => 'left_ring',
      FingerPosition.leftLittle => 'left_little',
      FingerPosition.rightThumb => 'right_thumb',
      FingerPosition.rightIndex => 'right_index',
      FingerPosition.rightMiddle => 'right_middle',
      FingerPosition.rightRing => 'right_ring',
      FingerPosition.rightLittle => 'right_little',
    };

enum FingerHand { leftFour, rightFour, twoThumbs }

String fingerHandName(FingerHand h) => switch (h) {
      FingerHand.leftFour => 'left_four',
      FingerHand.rightFour => 'right_four',
      FingerHand.twoThumbs => 'two_thumbs',
    };

/// Single finger capture (raw + WSQ + quality).
@immutable
class FingerCapture {
  final Uint8List rawBytes; // 8-bit grayscale
  final int width;
  final int height;
  final int quality; // 0-100 FBI IQS
  final Uint8List? wsqBytes;
  final String? isoBase64; // ISO 19794-2 if enrolled

  const FingerCapture({
    required this.rawBytes,
    required this.width,
    required this.height,
    required this.quality,
    this.wsqBytes,
    this.isoBase64,
  });

  bool get isQualityPass => quality >= 40;
}

/// Slap split result (4 fingers or 2 thumbs).
@immutable
class SlapCapture {
  final FingerHand hand;
  final List<FingerCapture> fingers;

  const SlapCapture({required this.hand, required this.fingers});
}

/// Enrollment templates bundle (ISO 19794-2 per finger).
@immutable
class FingerEnrollment {
  final List<Uint8List> isoTemplates; // each 1024 bytes
  final int galleryId;

  const FingerEnrollment({required this.isoTemplates, required this.galleryId});
}

/// 1:N identification result.
@immutable
class FingerMatch {
  final int matchedId; // -1 if no match
  final double score; // 0-100, threshold 40

  const FingerMatch({required this.matchedId, required this.score});

  bool get isMatch => matchedId >= 0 && score > 40;
}

/// Events emitted by a [FingerprintTransport].
@immutable
sealed class FingerEvent {
  const FingerEvent();
}

class FingerConnected extends FingerEvent {
  final String deviceId;
  const FingerConnected(this.deviceId);
}

class FingerDisconnected extends FingerEvent {
  const FingerDisconnected();
}

class FingerCaptured extends FingerEvent {
  final FingerCapture capture;
  const FingerCaptured(this.capture);
}

class SlapCaptured extends FingerEvent {
  final SlapCapture slap;
  const SlapCaptured(this.slap);
}

class FingerEnrolled extends FingerEvent {
  final FingerEnrollment enrollment;
  const FingerEnrolled(this.enrollment);
}

class FingerIdentified extends FingerEvent {
  final FingerMatch match;
  const FingerIdentified(this.match);
}

class FingerError extends FingerEvent {
  final String message;
  const FingerError(this.message);
}

/// Raw transport for a FAP60 reader.
abstract class FingerprintTransport {
  Stream<FingerEvent> get events;
  Future<void> connect();
  Future<void> disconnect();
  Future<void> captureFlat();
  Future<void> captureSlap({required FingerHand hand});
  Future<void> captureRoll();
  Future<void> enroll({int fingerCount = 4});
  Future<void> identify();
  Future<void> setDryWet(int level);
  Future<void> beep(int type);
  Future<void> dispose();
}

/// Platform-channel transport speaking to the native FAP60 bridge.
class PlatformChannelFingerprintTransport implements FingerprintTransport {
  PlatformChannelFingerprintTransport({String channel = 'eel/fingerprint'})
      : _channel = MethodChannel(channel) {
    _channel.setMethodCallHandler(_onMethodCall);
  }

  final MethodChannel _channel;
  final _controller = StreamController<FingerEvent>.broadcast();

  @override
  Stream<FingerEvent> get events => _controller.stream;

  Future<dynamic> _onMethodCall(MethodCall call) async {
    switch (call.method) {
      case 'connected':
        _controller.add(FingerConnected(call.arguments as String? ?? 'fap60'));
      case 'disconnected':
        _controller.add(const FingerDisconnected());
      case 'error':
        _controller.add(FingerError(call.arguments as String? ?? 'fingerprint error'));
      case 'captured':
        _controller.add(FingerCaptured(_captureFromMap(call.arguments as Map)));
      case 'slapCaptured':
        final args = call.arguments as Map;
        final hand = fingerHandFromName(args['hand'] as String? ?? 'left_four');
        final list = (args['fingers'] as List).cast<Map>();
        _controller.add(SlapCaptured(SlapCapture(
          hand: hand,
          fingers: list.map(_captureFromMap).toList(),
        )));
      case 'enrolled':
        final args = call.arguments as Map;
        final templates = (args['templates'] as List)
            .map((e) => base64Decode(e as String))
            .toList();
        _controller.add(FingerEnrolled(FingerEnrollment(
          isoTemplates: templates,
          galleryId: args['gallery_id'] as int? ?? 0,
        )));
      case 'identified':
        final args = call.arguments as Map;
        _controller.add(FingerIdentified(FingerMatch(
          matchedId: args['matched_id'] as int? ?? -1,
          score: (args['score'] as num?)?.toDouble() ?? 0,
        )));
    }
  }

  @override
  Future<void> connect() => _channel.invokeMethod('connect');

  @override
  Future<void> disconnect() => _channel.invokeMethod('disconnect');

  @override
  Future<void> captureFlat() => _channel.invokeMethod('captureFlat');

  @override
  Future<void> captureSlap({required FingerHand hand}) =>
      _channel.invokeMethod('captureSlap', {'hand': fingerHandName(hand)});

  @override
  Future<void> captureRoll() => _channel.invokeMethod('captureRoll');

  @override
  Future<void> enroll({int fingerCount = 4}) =>
      _channel.invokeMethod('enroll', {'count': fingerCount});

  @override
  Future<void> identify() => _channel.invokeMethod('identify');

  @override
  Future<void> setDryWet(int level) =>
      _channel.invokeMethod('setDryWet', {'level': level});

  @override
  Future<void> beep(int type) =>
      _channel.invokeMethod('beep', {'type': type});

  @override
  Future<void> dispose() async {
    _channel.setMethodCallHandler(null);
    await _controller.close();
  }
}

FingerHand fingerHandFromName(String name) => switch (name) {
      'right_four' => FingerHand.rightFour,
      'two_thumbs' => FingerHand.twoThumbs,
      _ => FingerHand.leftFour,
    };

FingerCapture _captureFromMap(Map args) => FingerCapture(
      rawBytes: base64Decode(args['raw_base64'] as String),
      width: args['width'] as int? ?? 300,
      height: args['height'] as int? ?? 400,
      quality: args['quality'] as int? ?? 0,
      wsqBytes: args['wsq_base64'] != null ? base64Decode(args['wsq_base64'] as String) : null,
      isoBase64: args['iso_base64'] as String?,
    );

/// Simulated transport for tests and demos; never touches a platform channel.
class SimulatedFingerprintTransport implements FingerprintTransport {
  SimulatedFingerprintTransport();

  final _controller = StreamController<FingerEvent>.broadcast();
  int _galleryCounter = 0;

  @override
  Stream<FingerEvent> get events => _controller.stream;

  @override
  Future<void> connect() async {
    _controller.add(const FingerConnected('SIM-FAP60-01'));
  }

  @override
  Future<void> disconnect() async {
    _controller.add(const FingerDisconnected());
  }

  @override
  Future<void> captureFlat() async {
    await Future<void>.delayed(const Duration(milliseconds: 50));
    _controller.add(FingerCaptured(_fakeCapture(300, 400)));
  }

  @override
  Future<void> captureSlap({required FingerHand hand}) async {
    await Future<void>.delayed(const Duration(milliseconds: 80));
    final count = hand == FingerHand.twoThumbs ? 2 : 4;
    _controller.add(SlapCaptured(SlapCapture(
      hand: hand,
      fingers: List.generate(count, (_) => _fakeCapture(256, 360)),
    )));
  }

  @override
  Future<void> captureRoll() async {
    await Future<void>.delayed(const Duration(milliseconds: 80));
    _controller.add(FingerCaptured(_fakeCapture(800, 750)));
  }

  @override
  Future<void> enroll({int fingerCount = 4}) async {
    await Future<void>.delayed(const Duration(milliseconds: 100));
    _galleryCounter++;
    _controller.add(FingerEnrolled(FingerEnrollment(
      isoTemplates: List.generate(fingerCount, (_) => Uint8List(1024)),
      galleryId: _galleryCounter - 1,
    )));
  }

  @override
  Future<void> identify() async {
    await Future<void>.delayed(const Duration(milliseconds: 80));
    _controller.add(const FingerIdentified(FingerMatch(matchedId: 0, score: 88)));
  }

  @override
  Future<void> setDryWet(int level) async {}

  @override
  Future<void> beep(int type) async {}

  @override
  Future<void> dispose() async {
    await _controller.close();
  }

  FingerCapture _fakeCapture(int w, int h) => FingerCapture(
        rawBytes: Uint8List(w * h),
        width: w,
        height: h,
        quality: 88,
        wsqBytes: Uint8List(512),
        isoBase64: base64Encode(Uint8List(1024)),
      );
}

/// Stateful scanner facade over a transport (mirrors IrisScanner).
class FingerprintScanner extends ChangeNotifier {
  FingerprintScanner({FingerprintTransport? transport})
      : _transport = transport ?? PlatformChannelFingerprintTransport() {
    _sub = _transport.events.listen(_onEvent);
  }

  final FingerprintTransport _transport;
  late final StreamSubscription<FingerEvent> _sub;

  bool _connected = false;
  bool _busy = false;
  String? _error;
  FingerCapture? _lastCapture;
  SlapCapture? _lastSlap;
  FingerEnrollment? _lastEnrollment;
  FingerMatch? _lastMatch;

  bool get connected => _connected;
  bool get busy => _busy;
  String? get error => _error;
  FingerCapture? get lastCapture => _lastCapture;
  SlapCapture? get lastSlap => _lastSlap;
  FingerEnrollment? get lastEnrollment => _lastEnrollment;
  FingerMatch? get lastMatch => _lastMatch;

  /// Raw event stream (for drivers that need to await the next capture).
  Stream<FingerEvent> get events => _transport.events;

  void _onEvent(FingerEvent event) {
    switch (event) {
      case FingerConnected():
        _connected = true;
        _busy = false;
        _error = null;
      case FingerDisconnected():
        _connected = false;
      case FingerCaptured(:final capture):
        _lastCapture = capture;
        _busy = false;
      case SlapCaptured(:final slap):
        _lastSlap = slap;
        _busy = false;
      case FingerEnrolled(:final enrollment):
        _lastEnrollment = enrollment;
        _busy = false;
      case FingerIdentified(:final match):
        _lastMatch = match;
        _busy = false;
      case FingerError(:final message):
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

  Future<void> captureFlat() async {
    _busy = true;
    notifyListeners();
    await _transport.captureFlat();
  }

  Future<void> captureSlap(FingerHand hand) async {
    _busy = true;
    notifyListeners();
    await _transport.captureSlap(hand: hand);
  }

  Future<void> captureRoll() async {
    _busy = true;
    notifyListeners();
    await _transport.captureRoll();
  }

  Future<void> enroll({int fingerCount = 4}) async {
    _busy = true;
    notifyListeners();
    await _transport.enroll(fingerCount: fingerCount);
  }

  Future<void> identify() async {
    _busy = true;
    notifyListeners();
    await _transport.identify();
  }

  Future<void> setDryWet(int level) => _transport.setDryWet(level);
  Future<void> beep(int type) => _transport.beep(type);

  @override
  void dispose() {
    _sub.cancel();
    _transport.dispose();
    super.dispose();
  }
}
