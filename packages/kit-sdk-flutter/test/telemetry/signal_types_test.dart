// ignore_for_file: avoid_redundant_argument_values

import 'package:flutter_test/flutter_test.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

void main() {
  const deviceId = 'test-device-001';
  final now = DateTime.utc(2026, 8, 22, 12, 0, 0);
  const traceId = 'trace-001';
  const spanId = 'span-001';
  const correlationId = 'corr-001';
  const prevHash = 'abc123';

  group('LocationSignal round-trip', () {
    test('serialises and deserialises', () {
      final s = LocationSignal(
        deviceId: deviceId, occurredAt: now, monotonicCounter: 1,
        traceId: traceId, spanId: spanId, correlationId: correlationId,
        prevHash: prevHash, latitudeDeg: -1.286, longitudeDeg: 36.817,
        accuracyM: 10.0, altitudeM: 1700.0,
        cellTowerId: 'tower-1', cellRssi: -80,
        wifiBssid: 'AA:BB:CC:DD:EE:FF', wifiRssi: -55,
        isCoarsened: false, thisHash: 'h1',
      );
      final j = s.toJson();
      final s2 = LocationSignal.fromJson(j);
      expect(s2.latitudeDeg, s.latitudeDeg);
      expect(s2.longitudeDeg, s.longitudeDeg);
      expect(s2.accuracyM, s.accuracyM);
      expect(s2.cellTowerId, s.cellTowerId);
      expect(s2.isCoarsened, false);
      expect(s2.signalType, 'location');
    });
  });

  group('TimeSignal round-trip', () {
    test('serialises and deserialises', () {
      final s = TimeSignal(
        deviceId: deviceId, occurredAt: now, monotonicCounter: 2,
        traceId: traceId, spanId: spanId, correlationId: correlationId,
        prevHash: prevHash, wallClockUtc: now, monotonicMs: 12345,
        hlcTimestamp: '2026-08-22T12:00:00.000Z+0',
        ntpOffsetMs: 12, lastTimeFenceCheckUtc: now,
        thisHash: 'h2',
      );
      final s2 = TimeSignal.fromJson(s.toJson());
      expect(s2.monotonicMs, 12345);
      expect(s2.ntpOffsetMs, 12);
      expect(s2.signalType, 'time');
    });
  });

  group('ActivitySignal round-trip', () {
    test('serialises and deserialises', () {
      final s = ActivitySignal(
        deviceId: deviceId, occurredAt: now, monotonicCounter: 3,
        traceId: traceId, spanId: spanId, correlationId: correlationId,
        prevHash: prevHash, activityKind: 'workflow_step',
        workflowStep: 'capture_biometric', thisHash: 'h3',
      );
      final s2 = ActivitySignal.fromJson(s.toJson());
      expect(s2.activityKind, 'workflow_step');
      expect(s2.workflowStep, 'capture_biometric');
      expect(s2.signalType, 'activity');
    });
  });

  group('ProximalRfidSignal round-trip', () {
    test('serialises and deserialises', () {
      final s = ProximalRfidSignal(
        deviceId: deviceId, occurredAt: now, monotonicCounter: 4,
        traceId: traceId, spanId: spanId, correlationId: correlationId,
        prevHash: prevHash, rfidEpc: 'EPC-0001', rssi: -70,
        sealTagStatus: 'sealed', boxBodyId: 'BOX-001', thisHash: 'h4',
      );
      final s2 = ProximalRfidSignal.fromJson(s.toJson());
      expect(s2.rfidEpc, 'EPC-0001');
      expect(s2.sealTagStatus, 'sealed');
      expect(s2.signalType, 'proximal_rfid');
    });
  });

  group('MovementSignal round-trip', () {
    test('serialises and deserialises', () {
      final s = MovementSignal(
        deviceId: deviceId, occurredAt: now, monotonicCounter: 5,
        traceId: traceId, spanId: spanId, correlationId: correlationId,
        prevHash: prevHash,
        accelX: 0.1, accelY: 0.2, accelZ: 9.8,
        gyroX: 0.01, gyroY: 0.02, gyroZ: 0.03,
        motionClass: MotionClass.walking, isDownsampled: true,
        thisHash: 'h5',
      );
      final s2 = MovementSignal.fromJson(s.toJson());
      expect(s2.accelZ, 9.8);
      expect(s2.motionClass, MotionClass.walking);
      expect(s2.isDownsampled, true);
      expect(s2.signalType, 'movement');
    });
  });

  group('LoginLogoutSignal round-trip', () {
    test('serialises and deserialises', () {
      final s = LoginLogoutSignal(
        deviceId: deviceId, occurredAt: now, monotonicCounter: 6,
        traceId: traceId, spanId: spanId, correlationId: correlationId,
        prevHash: prevHash, actorSvid: 'spiffe://eel/officer/1234',
        sessionEvent: SessionEvent.login,
        biometricResult: 'pass', thisHash: 'h6',
      );
      final s2 = LoginLogoutSignal.fromJson(s.toJson());
      expect(s2.actorSvid, 'spiffe://eel/officer/1234');
      expect(s2.sessionEvent, SessionEvent.login);
      expect(s2.signalType, 'login_logout');
    });
  });

  group('ReadSignal round-trip', () {
    test('serialises and deserialises', () {
      final s = ReadSignal(
        deviceId: deviceId, occurredAt: now, monotonicCounter: 7,
        traceId: traceId, spanId: spanId, correlationId: correlationId,
        prevHash: prevHash, entityKind: 'voter_record',
        entityIdHash: 'sha256hash', thisHash: 'h7',
      );
      final s2 = ReadSignal.fromJson(s.toJson());
      expect(s2.entityKind, 'voter_record');
      expect(s2.entityIdHash, 'sha256hash');
      expect(s2.signalType, 'read');
    });
  });

  group('WriteSignal round-trip', () {
    test('serialises and deserialises', () {
      final s = WriteSignal(
        deviceId: deviceId, occurredAt: now, monotonicCounter: 8,
        traceId: traceId, spanId: spanId, correlationId: correlationId,
        prevHash: prevHash, entityKind: 'enrolment',
        entityIdHash: 'sha256hash2', thisHash: 'h8',
      );
      final s2 = WriteSignal.fromJson(s.toJson());
      expect(s2.entityKind, 'enrolment');
      expect(s2.signalType, 'write');
    });
  });

  group('BatterySignal round-trip', () {
    test('serialises and deserialises', () {
      final s = BatterySignal(
        deviceId: deviceId, occurredAt: now, monotonicCounter: 9,
        traceId: traceId, spanId: spanId, correlationId: correlationId,
        prevHash: prevHash, percentInt: 42,
        chargingState: ChargingState.discharging,
        temperatureCelsius: 35.5, thisHash: 'h9',
      );
      final s2 = BatterySignal.fromJson(s.toJson());
      expect(s2.percentInt, 42);
      expect(s2.chargingState, ChargingState.discharging);
      expect(s2.signalType, 'battery');
    });
  });

  group('LockUnlockSignal round-trip', () {
    test('serialises and deserialises', () {
      final s = LockUnlockSignal(
        deviceId: deviceId, occurredAt: now, monotonicCounter: 10,
        traceId: traceId, spanId: spanId, correlationId: correlationId,
        prevHash: prevHash, lockEvent: LockEvent.biometricUnlock,
        thisHash: 'h10',
      );
      final s2 = LockUnlockSignal.fromJson(s.toJson());
      expect(s2.lockEvent, LockEvent.biometricUnlock);
      expect(s2.signalType, 'lock_unlock');
    });
  });

  group('DropSignal round-trip', () {
    test('serialises and deserialises', () {
      final s = DropSignal(
        deviceId: deviceId, occurredAt: now, monotonicCounter: 11,
        traceId: traceId, spanId: spanId, correlationId: correlationId,
        prevHash: prevHash, peakAccelG: 5.2, windowMs: 200,
        freeFallDetected: true, severity: SignalSeverity.critical,
        thisHash: 'h11',
      );
      final s2 = DropSignal.fromJson(s.toJson());
      expect(s2.peakAccelG, 5.2);
      expect(s2.freeFallDetected, true);
      expect(s2.severity, SignalSeverity.critical);
      expect(s2.signalType, 'drop');
    });
  });

  group('NetworkSignal round-trip', () {
    test('serialises and deserialises', () {
      final s = NetworkSignal(
        deviceId: deviceId, occurredAt: now, monotonicCounter: 12,
        traceId: traceId, spanId: spanId, correlationId: correlationId,
        prevHash: prevHash, connectivityClass: ConnectivityClass.wifi,
        effectiveBandwidthKbps: 10000, latencyMs: 25, thisHash: 'h12',
      );
      final s2 = NetworkSignal.fromJson(s.toJson());
      expect(s2.connectivityClass, ConnectivityClass.wifi);
      expect(s2.effectiveBandwidthKbps, 10000);
      expect(s2.signalType, 'network');
    });
  });

  group('AttestationSignal round-trip', () {
    test('serialises and deserialises', () {
      final s = AttestationSignal(
        deviceId: deviceId, occurredAt: now, monotonicCounter: 13,
        traceId: traceId, spanId: spanId, correlationId: correlationId,
        prevHash: prevHash, attestationStatus: 'play_integrity_pass',
        kitCertValid: true,
        nextCertRefreshUtc: now.add(const Duration(days: 30)),
        attestationProvider: 'play_integrity', thisHash: 'h13',
      );
      final s2 = AttestationSignal.fromJson(s.toJson());
      expect(s2.attestationStatus, 'play_integrity_pass');
      expect(s2.kitCertValid, true);
      expect(s2.attestationProvider, 'play_integrity');
      expect(s2.signalType, 'attestation');
    });
  });
}
