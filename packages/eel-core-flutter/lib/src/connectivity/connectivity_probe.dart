import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:flutter/foundation.dart';

/// Connectivity states (plan §2.6).
enum ConnectivityState { online, degraded, offline, captivePortal, untrusted }

@immutable
class ConnectivitySnapshot {
  const ConnectivitySnapshot({
    required this.state,
    this.probedAt,
    this.detail,
  });

  final ConnectivityState state;
  final DateTime? probedAt;
  final String? detail;

  bool get isUsable => state == ConnectivityState.online || state == ConnectivityState.degraded;

  ConnectivitySnapshot copyWith({
    ConnectivityState? state,
    DateTime? probedAt,
    String? detail,
  }) =>
      ConnectivitySnapshot(
        state: state ?? this.state,
        probedAt: probedAt ?? this.probedAt,
        detail: detail ?? this.detail,
      );
}

/// Pluggable network reachability (connectivity_plus at the app layer).
abstract class ReachabilitySource {
  /// Current coarse state: none / mobile / wifi / ethernet.
  Future<String> current();
}

/// Probe result reported to the UI.
typedef ProbeFn = Future<ConnectivityState> Function();

/// Captive-portal + untrusted-TLS probe (plan §2.6):
/// - HEAD {bff}/healthz through the pinned client.
/// - TLS pin failure → untrusted (zero tokens emitted).
/// - Pin/DNS failure pattern → captivePortal.
class ConnectivityProbe {
  ConnectivityProbe({
    required this.healthPath,
    required this.probe,
    this.reachability,
    this.degradedMbps = 0.05,
  });

  /// e.g. `/api/v1/mobile/bff/healthz`.
  final String healthPath;
  final ProbeFn probe;
  final Reachability? reachability;
  final double degradedMbps;

  Future<ConnectivitySnapshot> evaluate() async {
    final transport = await reachability?.current() ?? 'unknown';
    if (transport == 'none') {
      return ConnectivitySnapshot(
        state: ConnectivityState.offline,
        probedAt: DateTime.now(),
      );
    }
    final state = await probe();
    return ConnectivitySnapshot(
      state: state,
      probedAt: DateTime.now(),
      detail: transport,
    );
  }
}

/// Abstract reachability for tests.
abstract class Reachability implements ReachabilitySource {}

/// Maps a thrown probe error to a connectivity state (plan §2.6 table).
ConnectivityState stateFromProbeError(Object error) {
  final message = error.toString().toLowerCase();
  if (message.contains('certificate') ||
      message.contains('handshake') ||
      message.contains('pin')) {
    return ConnectivityState.untrusted;
  }
  if (message.contains('redirect') ||
      message.contains('302') ||
      message.contains('307') ||
      message.contains('dns')) {
    return ConnectivityState.captivePortal;
  }
  return ConnectivityState.offline;
}