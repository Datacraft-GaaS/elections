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

/// Abstract signal base and sink contract for the eel telemetry layer.
library;

import 'dart:async';
import 'package:uuid/uuid.dart';

const _uuid = Uuid();

/// Generates a new trace-ID (UUID v4 string).
String newTraceId() => _uuid.v4();

/// Generates a new span-ID (UUID v4 string).
String newSpanId() => _uuid.v4();

/// Severity of a telemetry signal.
enum SignalSeverity { info, warning, critical }

/// Base class for all 13 telemetry signal classes.
///
/// Every signal carries:
/// - [deviceId] — the emitting device identifier (SVID-derived).
/// - [occurredAt] — wall-clock timestamp at emission.
/// - [monotonicCounter] — per-device monotonic sequence number; server rejects
///   out-of-order signals.
/// - [traceId] / [spanId] / [correlationId] — W3C traceparent fields.
/// - [prevHash] — BLAKE3/SHA-256 of the previous signal in the per-device chain.
/// - [thisHash] — BLAKE3/SHA-256 of this signal's canonical bytes.
/// - [signature] — optional HSM/HMAC signature populated by [Signer].
abstract class Signal {
  Signal({
    required this.deviceId,
    required this.occurredAt,
    required this.monotonicCounter,
    required this.traceId,
    required this.spanId,
    required this.correlationId,
    required this.prevHash,
    this.thisHash = '',
    this.signature,
    this.severity = SignalSeverity.info,
  });

  final String deviceId;
  final DateTime occurredAt;
  final int monotonicCounter;
  final String traceId;
  final String spanId;
  final String correlationId;
  final String prevHash;

  /// Populated after hashing; empty string until [HashChain.attach] runs.
  String thisHash;

  /// Populated by [Signer] before dispatch; null for unsigned signals.
  String? signature;

  final SignalSeverity severity;

  /// Signal class name used as the discriminator in serialisation.
  String get signalType;

  /// Serialise to JSON for storage and egress.
  Map<String, Object?> toJson();

  /// Factory stub — concrete classes provide their own fromJson.
  static Signal fromJson(Map<String, Object?> json) {
    throw UnimplementedError('Call the concrete class fromJson instead.');
  }

  /// Fields common to all signals; concrete classes merge their own fields.
  Map<String, Object?> baseJson() => {
        'signalType': signalType,
        'deviceId': deviceId,
        'occurredAt': occurredAt.toUtc().toIso8601String(),
        'monotonicCounter': monotonicCounter,
        'traceId': traceId,
        'spanId': spanId,
        'correlationId': correlationId,
        'prevHash': prevHash,
        'thisHash': thisHash,
        'signature': signature,
        'severity': severity.name,
      };
}

/// Contract for anything that can accept and flush signals.
abstract class SignalSink {
  /// Accept a single signal for buffering or immediate dispatch.
  Future<void> accept(Signal signal);

  /// Flush buffered signals to the upstream sink / server.
  Future<void> flush();

  /// Release resources.
  Future<void> close();
}
