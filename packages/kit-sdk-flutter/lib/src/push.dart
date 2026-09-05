import 'package:flutter/foundation.dart';

/// Push notification payload shape (plan §3.5).
///
/// Payloads carry **only** event type + record id; no PII. Content is fetched
/// authenticated on tap (FR-M30-025, FR-M33-005).
@immutable
class PushPayload {
  const PushPayload({
    required this.eventType,
    required this.recordId,
    this.timestamp,
  });

  /// CloudEvents `type`, e.g. `eel.field_mobile.incident_reported.v1`.
  final String eventType;

  /// Record id to fetch content for on tap.
  final String recordId;

  /// Server timestamp of the event (informational; client clock is not
  /// authoritative per §2.3).
  final DateTime? timestamp;

  Map<String, dynamic> toData() => {
        'event_type': eventType,
        'record_id': recordId,
        if (timestamp != null) 'timestamp': timestamp!.toUtc().toIso8601String(),
      };

  factory PushPayload.fromJson(Map<String, dynamic> json) => PushPayload(
        eventType: json['event_type'] as String? ?? '',
        recordId: json['record_id'] as String? ?? '',
        timestamp: json['timestamp'] != null
            ? DateTime.parse(json['timestamp'] as String)
            : null,
      );

  @override
  bool operator ==(Object other) =>
      other is PushPayload &&
      other.eventType == eventType &&
      other.recordId == recordId;

  @override
  int get hashCode => Object.hash(eventType, recordId);
}

/// Push notification registration state.
enum PushState {
  /// No token registered yet.
  unregistered,

  /// Token registered and valid.
  registered,

  /// Token rejected by server (410 Gone — re-register on foreground).
  expired,

  /// Registration failed (e.g. platform error).
  failed,
}

/// Push notification abstraction (plan §3.5).
///
/// Apps bind a real implementation (FCM/APNs); tests bind [InMemoryPushClient].
abstract class PushClient {
  /// Current registration state.
  PushState get state;

  /// Current push token (null if unregistered).
  String? get token;

  /// Register for push notifications. Returns the token.
  Future<String> register();

  /// Unregister push notifications.
  Future<void> unregister();

  /// Re-register if state is [PushState.expired] (EC-MOB-05).
  /// Returns true if re-registration was needed and succeeded.
  Future<bool> reRegisterIfNeeded();
}

/// In-memory push client for tests.
class InMemoryPushClient implements PushClient {
  PushState _state = PushState.unregistered;
  String? _token;

  @override
  PushState get state => _state;

  @override
  String? get token => _token;

  @override
  Future<String> register() async {
    _token = 'test-token-${DateTime.now().millisecondsSinceEpoch}';
    _state = PushState.registered;
    return _token!;
  }

  @override
  Future<void> unregister() async {
    _token = null;
    _state = PushState.unregistered;
  }

  @override
  Future<bool> reRegisterIfNeeded() async {
    if (_state == PushState.expired) {
      await register();
      return true;
    }
    return false;
  }

  void setExpired() => _state = PushState.expired;

  void reset() {
    _state = PushState.unregistered;
    _token = null;
  }
}

/// Push payload validator (plan §3.5: no PII).
///
/// Ensures payloads conform to the allowed schema before forwarding to
/// FCM/APNs. Returns null if the payload is valid, or an error message if
/// not.
String? validatePushPayload(PushPayload payload) {
  if (payload.eventType.isEmpty) {
    return 'event_type is required';
  }
  if (payload.recordId.isEmpty) {
    return 'record_id is required';
  }
  // Check for PII patterns in eventType (defensive).
  if (payload.eventType.contains(RegExp(r'[0-9]{9,}'))) {
    return 'event_type contains potential PII (numeric ID)';
  }
  return null;
}

/// Push token registry entry (plan §4.1: `POST /api/v1/mobile/bff/push-tokens`).
@immutable
class PushTokenEntry {
  const PushTokenEntry({
    required this.deviceId,
    required this.token,
    required this.platform,
    required this.registeredAt,
    this.expiresAt,
  });

  final String deviceId;
  final String token;
  final String platform; // 'fcm' or 'apns'
  final DateTime registeredAt;
  final DateTime? expiresAt;

  Map<String, dynamic> toJson() => {
        'device_id': deviceId,
        'token': token,
        'platform': platform,
        'registered_at': registeredAt.toUtc().toIso8601String(),
        if (expiresAt != null) 'expires_at': expiresAt!.toUtc().toIso8601String(),
      };

  factory PushTokenEntry.fromJson(Map<String, dynamic> json) => PushTokenEntry(
        deviceId: json['device_id'] as String? ?? '',
        token: json['token'] as String? ?? '',
        platform: json['platform'] as String? ?? 'fcm',
        registeredAt: DateTime.parse(json['registered_at'] as String),
        expiresAt: json['expires_at'] != null
            ? DateTime.parse(json['expires_at'] as String)
            : null,
      );
}
