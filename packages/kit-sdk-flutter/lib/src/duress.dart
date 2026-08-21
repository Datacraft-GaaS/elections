import 'offline_queue.dart';

/// Duress template hook.
///
/// On pattern match, emits a covert KitDuressAlert event via the offline
/// queue while keeping the UI indistinguishable from normal operation.
class EelDuressHook {
  EelDuressHook({
    required EelOfflineQueue queue,
    required bool Function(String input) matcher,
    required String Function() deviceIdProvider,
  })  : _queue = queue,
        _matcher = matcher,
        _deviceIdProvider = deviceIdProvider;

  final EelOfflineQueue _queue;
  final bool Function(String input) _matcher;
  final String Function() _deviceIdProvider;

  /// Check [input] against the duress pattern.
  ///
  /// The caller must treat true and false branches identically in UI to avoid
  /// side-channel leakage.
  Future<bool> check(String input) async {
    final matched = _matcher(input);
    if (matched) {
      await _emitAlert();
    }
    return matched;
  }

  Future<void> _emitAlert() async {
    final ts = DateTime.now().millisecondsSinceEpoch;
    await _queue.enqueue(
      id: '_duress_$ts',
      idempotencyKey: 'duress_${_deviceIdProvider()}_$ts',
      payload: {
        'type': 'KitDuressAlert',
        'device_id': _deviceIdProvider(),
        'ts': DateTime.fromMillisecondsSinceEpoch(ts).toIso8601String(),
      },
    );
  }
}
