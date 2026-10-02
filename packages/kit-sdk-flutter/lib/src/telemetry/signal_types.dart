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

/// Concrete Signal subclasses for all 13 telemetry signal classes.
library;

import 'signal_sink.dart';

// ---------------------------------------------------------------------------
// 1. LocationSignal
// ---------------------------------------------------------------------------

class LocationSignal extends Signal {
  LocationSignal({
    required super.deviceId,
    required super.occurredAt,
    required super.monotonicCounter,
    required super.traceId,
    required super.spanId,
    required super.correlationId,
    required super.prevHash,
    required this.latitudeDeg,
    required this.longitudeDeg,
    required this.accuracyM,
    this.altitudeM,
    this.cellTowerId,
    this.cellRssi,
    this.wifiBssid,
    this.wifiRssi,
    this.isCoarsened = false,
    super.thisHash,
    super.signature,
    super.severity,
  });

  final double latitudeDeg;
  final double longitudeDeg;
  final double accuracyM;
  final double? altitudeM;
  final String? cellTowerId;
  final int? cellRssi;
  final String? wifiBssid;
  final int? wifiRssi;

  /// True when accuracy > 500 m and position has been coarsened to district.
  final bool isCoarsened;

  @override
  String get signalType => 'location';

  @override
  Map<String, Object?> toJson() => {
        ...baseJson(),
        'latitudeDeg': latitudeDeg,
        'longitudeDeg': longitudeDeg,
        'accuracyM': accuracyM,
        'altitudeM': altitudeM,
        'cellTowerId': cellTowerId,
        'cellRssi': cellRssi,
        'wifiBssid': wifiBssid,
        'wifiRssi': wifiRssi,
        'isCoarsened': isCoarsened,
      };

  factory LocationSignal.fromJson(Map<String, Object?> j) => LocationSignal(
        deviceId: j['deviceId'] as String,
        occurredAt: DateTime.parse(j['occurredAt'] as String),
        monotonicCounter: j['monotonicCounter'] as int,
        traceId: j['traceId'] as String,
        spanId: j['spanId'] as String,
        correlationId: j['correlationId'] as String,
        prevHash: j['prevHash'] as String,
        latitudeDeg: (j['latitudeDeg'] as num).toDouble(),
        longitudeDeg: (j['longitudeDeg'] as num).toDouble(),
        accuracyM: (j['accuracyM'] as num).toDouble(),
        altitudeM: (j['altitudeM'] as num?)?.toDouble(),
        cellTowerId: j['cellTowerId'] as String?,
        cellRssi: j['cellRssi'] as int?,
        wifiBssid: j['wifiBssid'] as String?,
        wifiRssi: j['wifiRssi'] as int?,
        isCoarsened: j['isCoarsened'] as bool? ?? false,
        thisHash: j['thisHash'] as String? ?? '',
        signature: j['signature'] as String?,
        severity: SignalSeverity.values.byName(j['severity'] as String? ?? 'info'),
      );
}

// ---------------------------------------------------------------------------
// 2. TimeSignal
// ---------------------------------------------------------------------------

class TimeSignal extends Signal {
  TimeSignal({
    required super.deviceId,
    required super.occurredAt,
    required super.monotonicCounter,
    required super.traceId,
    required super.spanId,
    required super.correlationId,
    required super.prevHash,
    required this.wallClockUtc,
    required this.monotonicMs,
    required this.hlcTimestamp,
    this.ntpOffsetMs,
    this.lastTimeFenceCheckUtc,
    super.thisHash,
    super.signature,
    super.severity,
  });

  final DateTime wallClockUtc;
  final int monotonicMs;
  final String hlcTimestamp;
  final int? ntpOffsetMs;
  final DateTime? lastTimeFenceCheckUtc;

  @override
  String get signalType => 'time';

  @override
  Map<String, Object?> toJson() => {
        ...baseJson(),
        'wallClockUtc': wallClockUtc.toUtc().toIso8601String(),
        'monotonicMs': monotonicMs,
        'hlcTimestamp': hlcTimestamp,
        'ntpOffsetMs': ntpOffsetMs,
        'lastTimeFenceCheckUtc': lastTimeFenceCheckUtc?.toUtc().toIso8601String(),
      };

  factory TimeSignal.fromJson(Map<String, Object?> j) => TimeSignal(
        deviceId: j['deviceId'] as String,
        occurredAt: DateTime.parse(j['occurredAt'] as String),
        monotonicCounter: j['monotonicCounter'] as int,
        traceId: j['traceId'] as String,
        spanId: j['spanId'] as String,
        correlationId: j['correlationId'] as String,
        prevHash: j['prevHash'] as String,
        wallClockUtc: DateTime.parse(j['wallClockUtc'] as String),
        monotonicMs: j['monotonicMs'] as int,
        hlcTimestamp: j['hlcTimestamp'] as String,
        ntpOffsetMs: j['ntpOffsetMs'] as int?,
        lastTimeFenceCheckUtc: (j['lastTimeFenceCheckUtc'] as String?) != null
            ? DateTime.parse(j['lastTimeFenceCheckUtc'] as String)
            : null,
        thisHash: j['thisHash'] as String? ?? '',
        signature: j['signature'] as String?,
        severity: SignalSeverity.values.byName(j['severity'] as String? ?? 'info'),
      );
}

// ---------------------------------------------------------------------------
// 3. ActivitySignal
// ---------------------------------------------------------------------------

class ActivitySignal extends Signal {
  ActivitySignal({
    required super.deviceId,
    required super.occurredAt,
    required super.monotonicCounter,
    required super.traceId,
    required super.spanId,
    required super.correlationId,
    required super.prevHash,
    required this.activityKind,
    this.screenName,
    this.workflowStep,
    this.keyboardOpen,
    this.cameraActive,
    super.thisHash,
    super.signature,
    super.severity,
  });

  /// e.g. 'screen_transition', 'workflow_step', 'keyboard_open', 'camera_active'.
  final String activityKind;
  final String? screenName;
  final String? workflowStep;
  final bool? keyboardOpen;
  final bool? cameraActive;

  @override
  String get signalType => 'activity';

  @override
  Map<String, Object?> toJson() => {
        ...baseJson(),
        'activityKind': activityKind,
        'screenName': screenName,
        'workflowStep': workflowStep,
        'keyboardOpen': keyboardOpen,
        'cameraActive': cameraActive,
      };

  factory ActivitySignal.fromJson(Map<String, Object?> j) => ActivitySignal(
        deviceId: j['deviceId'] as String,
        occurredAt: DateTime.parse(j['occurredAt'] as String),
        monotonicCounter: j['monotonicCounter'] as int,
        traceId: j['traceId'] as String,
        spanId: j['spanId'] as String,
        correlationId: j['correlationId'] as String,
        prevHash: j['prevHash'] as String,
        activityKind: j['activityKind'] as String,
        screenName: j['screenName'] as String?,
        workflowStep: j['workflowStep'] as String?,
        keyboardOpen: j['keyboardOpen'] as bool?,
        cameraActive: j['cameraActive'] as bool?,
        thisHash: j['thisHash'] as String? ?? '',
        signature: j['signature'] as String?,
        severity: SignalSeverity.values.byName(j['severity'] as String? ?? 'info'),
      );
}

// ---------------------------------------------------------------------------
// 4. ProximalRfidSignal
// ---------------------------------------------------------------------------

class ProximalRfidSignal extends Signal {
  ProximalRfidSignal({
    required super.deviceId,
    required super.occurredAt,
    required super.monotonicCounter,
    required super.traceId,
    required super.spanId,
    required super.correlationId,
    required super.prevHash,
    required this.rfidEpc,
    required this.rssi,
    this.sealTagStatus,
    this.boxBodyId,
    super.thisHash,
    super.signature,
    super.severity,
  });

  final String rfidEpc;
  final int rssi;
  final String? sealTagStatus;
  final String? boxBodyId;

  @override
  String get signalType => 'proximal_rfid';

  @override
  Map<String, Object?> toJson() => {
        ...baseJson(),
        'rfidEpc': rfidEpc,
        'rssi': rssi,
        'sealTagStatus': sealTagStatus,
        'boxBodyId': boxBodyId,
      };

  factory ProximalRfidSignal.fromJson(Map<String, Object?> j) => ProximalRfidSignal(
        deviceId: j['deviceId'] as String,
        occurredAt: DateTime.parse(j['occurredAt'] as String),
        monotonicCounter: j['monotonicCounter'] as int,
        traceId: j['traceId'] as String,
        spanId: j['spanId'] as String,
        correlationId: j['correlationId'] as String,
        prevHash: j['prevHash'] as String,
        rfidEpc: j['rfidEpc'] as String,
        rssi: j['rssi'] as int,
        sealTagStatus: j['sealTagStatus'] as String?,
        boxBodyId: j['boxBodyId'] as String?,
        thisHash: j['thisHash'] as String? ?? '',
        signature: j['signature'] as String?,
        severity: SignalSeverity.values.byName(j['severity'] as String? ?? 'info'),
      );
}

// ---------------------------------------------------------------------------
// 5. MovementSignal
// ---------------------------------------------------------------------------

enum MotionClass { stationary, walking, running, vehicle }

class MovementSignal extends Signal {
  MovementSignal({
    required super.deviceId,
    required super.occurredAt,
    required super.monotonicCounter,
    required super.traceId,
    required super.spanId,
    required super.correlationId,
    required super.prevHash,
    required this.accelX,
    required this.accelY,
    required this.accelZ,
    required this.gyroX,
    required this.gyroY,
    required this.gyroZ,
    this.magX,
    this.magY,
    this.magZ,
    this.motionClass,
    this.isDownsampled = false,
    super.thisHash,
    super.signature,
    super.severity,
  });

  final double accelX;
  final double accelY;
  final double accelZ;
  final double gyroX;
  final double gyroY;
  final double gyroZ;
  final double? magX;
  final double? magY;
  final double? magZ;
  final MotionClass? motionClass;
  final bool isDownsampled;

  @override
  String get signalType => 'movement';

  @override
  Map<String, Object?> toJson() => {
        ...baseJson(),
        'accelX': accelX,
        'accelY': accelY,
        'accelZ': accelZ,
        'gyroX': gyroX,
        'gyroY': gyroY,
        'gyroZ': gyroZ,
        'magX': magX,
        'magY': magY,
        'magZ': magZ,
        'motionClass': motionClass?.name,
        'isDownsampled': isDownsampled,
      };

  factory MovementSignal.fromJson(Map<String, Object?> j) => MovementSignal(
        deviceId: j['deviceId'] as String,
        occurredAt: DateTime.parse(j['occurredAt'] as String),
        monotonicCounter: j['monotonicCounter'] as int,
        traceId: j['traceId'] as String,
        spanId: j['spanId'] as String,
        correlationId: j['correlationId'] as String,
        prevHash: j['prevHash'] as String,
        accelX: (j['accelX'] as num).toDouble(),
        accelY: (j['accelY'] as num).toDouble(),
        accelZ: (j['accelZ'] as num).toDouble(),
        gyroX: (j['gyroX'] as num).toDouble(),
        gyroY: (j['gyroY'] as num).toDouble(),
        gyroZ: (j['gyroZ'] as num).toDouble(),
        magX: (j['magX'] as num?)?.toDouble(),
        magY: (j['magY'] as num?)?.toDouble(),
        magZ: (j['magZ'] as num?)?.toDouble(),
        motionClass: (j['motionClass'] as String?) != null
            ? MotionClass.values.byName(j['motionClass'] as String)
            : null,
        isDownsampled: j['isDownsampled'] as bool? ?? false,
        thisHash: j['thisHash'] as String? ?? '',
        signature: j['signature'] as String?,
        severity: SignalSeverity.values.byName(j['severity'] as String? ?? 'info'),
      );
}

// ---------------------------------------------------------------------------
// 6. LoginLogoutSignal
// ---------------------------------------------------------------------------

enum SessionEvent { login, logout }

class LoginLogoutSignal extends Signal {
  LoginLogoutSignal({
    required super.deviceId,
    required super.occurredAt,
    required super.monotonicCounter,
    required super.traceId,
    required super.spanId,
    required super.correlationId,
    required super.prevHash,
    required this.actorSvid,
    required this.sessionEvent,
    this.biometricResult,
    this.sessionDurationMs,
    super.thisHash,
    super.signature,
    super.severity,
  });

  final String actorSvid;
  final SessionEvent sessionEvent;
  final String? biometricResult;
  final int? sessionDurationMs;

  @override
  String get signalType => 'login_logout';

  @override
  Map<String, Object?> toJson() => {
        ...baseJson(),
        'actorSvid': actorSvid,
        'sessionEvent': sessionEvent.name,
        'biometricResult': biometricResult,
        'sessionDurationMs': sessionDurationMs,
      };

  factory LoginLogoutSignal.fromJson(Map<String, Object?> j) => LoginLogoutSignal(
        deviceId: j['deviceId'] as String,
        occurredAt: DateTime.parse(j['occurredAt'] as String),
        monotonicCounter: j['monotonicCounter'] as int,
        traceId: j['traceId'] as String,
        spanId: j['spanId'] as String,
        correlationId: j['correlationId'] as String,
        prevHash: j['prevHash'] as String,
        actorSvid: j['actorSvid'] as String,
        sessionEvent: SessionEvent.values.byName(j['sessionEvent'] as String),
        biometricResult: j['biometricResult'] as String?,
        sessionDurationMs: j['sessionDurationMs'] as int?,
        thisHash: j['thisHash'] as String? ?? '',
        signature: j['signature'] as String?,
        severity: SignalSeverity.values.byName(j['severity'] as String? ?? 'info'),
      );
}

// ---------------------------------------------------------------------------
// 7. ReadSignal
// ---------------------------------------------------------------------------

class ReadSignal extends Signal {
  ReadSignal({
    required super.deviceId,
    required super.occurredAt,
    required super.monotonicCounter,
    required super.traceId,
    required super.spanId,
    required super.correlationId,
    required super.prevHash,
    required this.entityKind,
    required this.entityIdHash,
    this.actorSvid,
    super.thisHash,
    super.signature,
    super.severity,
  });

  /// e.g. 'voter_record', 'kit_config', 'ota_update'.
  final String entityKind;

  /// SHA-256 hash of the raw entity ID — never the raw voter ID.
  final String entityIdHash;
  final String? actorSvid;

  @override
  String get signalType => 'read';

  @override
  Map<String, Object?> toJson() => {
        ...baseJson(),
        'entityKind': entityKind,
        'entityIdHash': entityIdHash,
        'actorSvid': actorSvid,
      };

  factory ReadSignal.fromJson(Map<String, Object?> j) => ReadSignal(
        deviceId: j['deviceId'] as String,
        occurredAt: DateTime.parse(j['occurredAt'] as String),
        monotonicCounter: j['monotonicCounter'] as int,
        traceId: j['traceId'] as String,
        spanId: j['spanId'] as String,
        correlationId: j['correlationId'] as String,
        prevHash: j['prevHash'] as String,
        entityKind: j['entityKind'] as String,
        entityIdHash: j['entityIdHash'] as String,
        actorSvid: j['actorSvid'] as String?,
        thisHash: j['thisHash'] as String? ?? '',
        signature: j['signature'] as String?,
        severity: SignalSeverity.values.byName(j['severity'] as String? ?? 'info'),
      );
}

// ---------------------------------------------------------------------------
// 8. WriteSignal
// ---------------------------------------------------------------------------

class WriteSignal extends Signal {
  WriteSignal({
    required super.deviceId,
    required super.occurredAt,
    required super.monotonicCounter,
    required super.traceId,
    required super.spanId,
    required super.correlationId,
    required super.prevHash,
    required this.entityKind,
    required this.entityIdHash,
    this.actorSvid,
    super.thisHash,
    super.signature,
    super.severity,
  });

  /// e.g. 'enrolment', 'identification', 'incident'.
  final String entityKind;
  final String entityIdHash;
  final String? actorSvid;

  @override
  String get signalType => 'write';

  @override
  Map<String, Object?> toJson() => {
        ...baseJson(),
        'entityKind': entityKind,
        'entityIdHash': entityIdHash,
        'actorSvid': actorSvid,
      };

  factory WriteSignal.fromJson(Map<String, Object?> j) => WriteSignal(
        deviceId: j['deviceId'] as String,
        occurredAt: DateTime.parse(j['occurredAt'] as String),
        monotonicCounter: j['monotonicCounter'] as int,
        traceId: j['traceId'] as String,
        spanId: j['spanId'] as String,
        correlationId: j['correlationId'] as String,
        prevHash: j['prevHash'] as String,
        entityKind: j['entityKind'] as String,
        entityIdHash: j['entityIdHash'] as String,
        actorSvid: j['actorSvid'] as String?,
        thisHash: j['thisHash'] as String? ?? '',
        signature: j['signature'] as String?,
        severity: SignalSeverity.values.byName(j['severity'] as String? ?? 'info'),
      );
}

// ---------------------------------------------------------------------------
// 9. BatterySignal
// ---------------------------------------------------------------------------

enum ChargingState { charging, discharging, full, unknown }

class BatterySignal extends Signal {
  BatterySignal({
    required super.deviceId,
    required super.occurredAt,
    required super.monotonicCounter,
    required super.traceId,
    required super.spanId,
    required super.correlationId,
    required super.prevHash,
    required this.percentInt,
    required this.chargingState,
    this.temperatureCelsius,
    this.estimatedTimeRemainingMs,
    this.usbPlugged,
    super.thisHash,
    super.signature,
    super.severity,
  });

  final int percentInt;
  final ChargingState chargingState;
  final double? temperatureCelsius;
  final int? estimatedTimeRemainingMs;
  final bool? usbPlugged;

  @override
  String get signalType => 'battery';

  @override
  Map<String, Object?> toJson() => {
        ...baseJson(),
        'percentInt': percentInt,
        'chargingState': chargingState.name,
        'temperatureCelsius': temperatureCelsius,
        'estimatedTimeRemainingMs': estimatedTimeRemainingMs,
        'usbPlugged': usbPlugged,
      };

  factory BatterySignal.fromJson(Map<String, Object?> j) => BatterySignal(
        deviceId: j['deviceId'] as String,
        occurredAt: DateTime.parse(j['occurredAt'] as String),
        monotonicCounter: j['monotonicCounter'] as int,
        traceId: j['traceId'] as String,
        spanId: j['spanId'] as String,
        correlationId: j['correlationId'] as String,
        prevHash: j['prevHash'] as String,
        percentInt: j['percentInt'] as int,
        chargingState: ChargingState.values.byName(j['chargingState'] as String),
        temperatureCelsius: (j['temperatureCelsius'] as num?)?.toDouble(),
        estimatedTimeRemainingMs: j['estimatedTimeRemainingMs'] as int?,
        usbPlugged: j['usbPlugged'] as bool?,
        thisHash: j['thisHash'] as String? ?? '',
        signature: j['signature'] as String?,
        severity: SignalSeverity.values.byName(j['severity'] as String? ?? 'info'),
      );
}

// ---------------------------------------------------------------------------
// 10. LockUnlockSignal
// ---------------------------------------------------------------------------

enum LockEvent { locked, unlocked, biometricUnlock, inactivityTimeout }

class LockUnlockSignal extends Signal {
  LockUnlockSignal({
    required super.deviceId,
    required super.occurredAt,
    required super.monotonicCounter,
    required super.traceId,
    required super.spanId,
    required super.correlationId,
    required super.prevHash,
    required this.lockEvent,
    super.thisHash,
    super.signature,
    super.severity,
  });

  final LockEvent lockEvent;

  @override
  String get signalType => 'lock_unlock';

  @override
  Map<String, Object?> toJson() => {
        ...baseJson(),
        'lockEvent': lockEvent.name,
      };

  factory LockUnlockSignal.fromJson(Map<String, Object?> j) => LockUnlockSignal(
        deviceId: j['deviceId'] as String,
        occurredAt: DateTime.parse(j['occurredAt'] as String),
        monotonicCounter: j['monotonicCounter'] as int,
        traceId: j['traceId'] as String,
        spanId: j['spanId'] as String,
        correlationId: j['correlationId'] as String,
        prevHash: j['prevHash'] as String,
        lockEvent: LockEvent.values.byName(j['lockEvent'] as String),
        thisHash: j['thisHash'] as String? ?? '',
        signature: j['signature'] as String?,
        severity: SignalSeverity.values.byName(j['severity'] as String? ?? 'info'),
      );
}

// ---------------------------------------------------------------------------
// 11. DropSignal
// ---------------------------------------------------------------------------

class DropSignal extends Signal {
  DropSignal({
    required super.deviceId,
    required super.occurredAt,
    required super.monotonicCounter,
    required super.traceId,
    required super.spanId,
    required super.correlationId,
    required super.prevHash,
    required this.peakAccelG,
    required this.windowMs,
    this.orientationChangeDeg,
    this.freeFallDetected = false,
    super.thisHash,
    super.signature,
    super.severity,
  });

  final double peakAccelG;
  final int windowMs;
  final double? orientationChangeDeg;
  final bool freeFallDetected;

  @override
  String get signalType => 'drop';

  @override
  Map<String, Object?> toJson() => {
        ...baseJson(),
        'peakAccelG': peakAccelG,
        'windowMs': windowMs,
        'orientationChangeDeg': orientationChangeDeg,
        'freeFallDetected': freeFallDetected,
      };

  factory DropSignal.fromJson(Map<String, Object?> j) => DropSignal(
        deviceId: j['deviceId'] as String,
        occurredAt: DateTime.parse(j['occurredAt'] as String),
        monotonicCounter: j['monotonicCounter'] as int,
        traceId: j['traceId'] as String,
        spanId: j['spanId'] as String,
        correlationId: j['correlationId'] as String,
        prevHash: j['prevHash'] as String,
        peakAccelG: (j['peakAccelG'] as num).toDouble(),
        windowMs: j['windowMs'] as int,
        orientationChangeDeg: (j['orientationChangeDeg'] as num?)?.toDouble(),
        freeFallDetected: j['freeFallDetected'] as bool? ?? false,
        thisHash: j['thisHash'] as String? ?? '',
        signature: j['signature'] as String?,
        severity: SignalSeverity.values.byName(j['severity'] as String? ?? 'critical'),
      );
}

// ---------------------------------------------------------------------------
// 12. NetworkSignal
// ---------------------------------------------------------------------------

enum ConnectivityClass { offline, g2, g3, g4, g5, wifi, unknown }

class NetworkSignal extends Signal {
  NetworkSignal({
    required super.deviceId,
    required super.occurredAt,
    required super.monotonicCounter,
    required super.traceId,
    required super.spanId,
    required super.correlationId,
    required super.prevHash,
    required this.connectivityClass,
    this.effectiveBandwidthKbps,
    this.latencyMs,
    this.retryBackoffState,
    super.thisHash,
    super.signature,
    super.severity,
  });

  final ConnectivityClass connectivityClass;
  final int? effectiveBandwidthKbps;
  final int? latencyMs;
  final String? retryBackoffState;

  @override
  String get signalType => 'network';

  @override
  Map<String, Object?> toJson() => {
        ...baseJson(),
        'connectivityClass': connectivityClass.name,
        'effectiveBandwidthKbps': effectiveBandwidthKbps,
        'latencyMs': latencyMs,
        'retryBackoffState': retryBackoffState,
      };

  factory NetworkSignal.fromJson(Map<String, Object?> j) => NetworkSignal(
        deviceId: j['deviceId'] as String,
        occurredAt: DateTime.parse(j['occurredAt'] as String),
        monotonicCounter: j['monotonicCounter'] as int,
        traceId: j['traceId'] as String,
        spanId: j['spanId'] as String,
        correlationId: j['correlationId'] as String,
        prevHash: j['prevHash'] as String,
        connectivityClass:
            ConnectivityClass.values.byName(j['connectivityClass'] as String),
        effectiveBandwidthKbps: j['effectiveBandwidthKbps'] as int?,
        latencyMs: j['latencyMs'] as int?,
        retryBackoffState: j['retryBackoffState'] as String?,
        thisHash: j['thisHash'] as String? ?? '',
        signature: j['signature'] as String?,
        severity: SignalSeverity.values.byName(j['severity'] as String? ?? 'info'),
      );
}

// ---------------------------------------------------------------------------
// 13. AttestationSignal
// ---------------------------------------------------------------------------

class AttestationSignal extends Signal {
  AttestationSignal({
    required super.deviceId,
    required super.occurredAt,
    required super.monotonicCounter,
    required super.traceId,
    required super.spanId,
    required super.correlationId,
    required super.prevHash,
    required this.attestationStatus,
    required this.kitCertValid,
    this.nextCertRefreshUtc,
    this.attestationProvider,
    super.thisHash,
    super.signature,
    super.severity,
  });

  /// e.g. 'play_integrity_pass', 'app_attest_pass', 'tpm_pass', 'fail'.
  final String attestationStatus;
  final bool kitCertValid;
  final DateTime? nextCertRefreshUtc;

  /// e.g. 'play_integrity', 'app_attest', 'tpm', 'tdx'.
  final String? attestationProvider;

  @override
  String get signalType => 'attestation';

  @override
  Map<String, Object?> toJson() => {
        ...baseJson(),
        'attestationStatus': attestationStatus,
        'kitCertValid': kitCertValid,
        'nextCertRefreshUtc': nextCertRefreshUtc?.toUtc().toIso8601String(),
        'attestationProvider': attestationProvider,
      };

  factory AttestationSignal.fromJson(Map<String, Object?> j) => AttestationSignal(
        deviceId: j['deviceId'] as String,
        occurredAt: DateTime.parse(j['occurredAt'] as String),
        monotonicCounter: j['monotonicCounter'] as int,
        traceId: j['traceId'] as String,
        spanId: j['spanId'] as String,
        correlationId: j['correlationId'] as String,
        prevHash: j['prevHash'] as String,
        attestationStatus: j['attestationStatus'] as String,
        kitCertValid: j['kitCertValid'] as bool,
        nextCertRefreshUtc: (j['nextCertRefreshUtc'] as String?) != null
            ? DateTime.parse(j['nextCertRefreshUtc'] as String)
            : null,
        attestationProvider: j['attestationProvider'] as String?,
        thisHash: j['thisHash'] as String? ?? '',
        signature: j['signature'] as String?,
        severity: SignalSeverity.values.byName(j['severity'] as String? ?? 'info'),
      );
}
