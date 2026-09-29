/// Duress alert service — silent beacon to war-room + BVI (M30 FR-026).
/// Uses EelHttpClient (mTLS via SPKI pinning) to send encrypted beacon.
library;
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

class DuressAlertService {
  final EelHttpClient? _client;

  DuressAlertService({EelHttpClient? client}) : _client = client;

  /// Send silent alert — never reveals action in UI.
  Future<void> sendSilentAlert({
    required String agentId,
    required String stationCode,
  }) async {
    if (_client == null) {
      throw StateError('DuressAlertService requires EelHttpClient (mTLS/SPKI). Per M30: hardware-gated when network unavailable.');
    }
    await _client!.post(
      '/api/v1/mobile/bff/duress',
      data: {
        'agent_id': agentId,
        'station_code': stationCode,
        'timestamp_utc': DateTime.now().toIso8601String(),
      },
    );
  }
}
