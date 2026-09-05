import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:flutter/foundation.dart';

/// Tamper event kinds (plan §2.4/§3.2).
enum TamperKind {
  rootDetected,
  emulatorDetected,
  debuggerAttached,
  fridaDetected,
  xposedDetected,
  resignDetected,
  screenCapture,
  unknown,
}

/// `eel.field_mobile.device_tamper_detected.v1` payload shape.
@immutable
class TamperEvent {
  const TamperEvent({
    required this.kind,
    required this.detectedAt,
    this.detail,
    this.checksPassed = const {},
  });

  final TamperKind kind;
  final DateTime detectedAt;
  final String? detail;

  /// Per-check pass/fail for the audit shadow (M28 AI monitoring).
  final Map<String, bool> checksPassed;

  String get cloudEventType =>
      'eel.field_mobile.device_tamper_detected.v1';

  Map<String, Object?> toData() => {
        'tamper_kind': kind.name,
        'detected_at': detectedAt.toUtc().toIso8601String(),
        if (detail != null) 'detail': detail,
        'checks': checksPassed,
      };
}

/// RASP check set (plan §3.2). Platform channels do the native probes; this
/// is the Dart-side model + reducer.
abstract class RaspinChecker {
  /// Returns per-check results; empty map on unsupported platform.
  Future<Map<String, bool>> runChecks();
}

/// Reduces RASP results to a tamper kind or null (all clear).
TamperKind? tamperKindFromChecks(Map<String, bool> checks) {
  if (checks['frida'] == false) return TamperKind.fridaDetected;
  if (checks['debugger'] == false) return TamperKind.debuggerAttached;
  if (checks['xposed'] == false) return TamperKind.xposedDetected;
  if (checks['resign'] == false) return TamperKind.resignDetected;
  if (checks['emulator'] == false) return TamperKind.emulatorDetected;
  if (checks['root'] == false) return TamperKind.rootDetected;
  return null;
}

/// Emits tamper events best-effort from a one-shot unpinned channel (§2.4).
abstract class TamperEventSink {
  Future<void> emit(TamperEvent event);
}

/// Composite: run checks, reduce, emit. Returns the kind if tampered.
Future<TamperKind?> detectAndReport({
  required RaspinChecker checker,
  required TamperEventSink sink,
  DateTime Function()? now,
}) async {
  final checks = await checker.runChecks();
  final kind = tamperKindFromChecks(checks);
  if (kind != null) {
    await sink.emit(
      TamperEvent(
        kind: kind,
        detectedAt: (now ?? DateTime.now)(),
        checksPassed: checks,
      ),
    );
  }
  return kind;
}