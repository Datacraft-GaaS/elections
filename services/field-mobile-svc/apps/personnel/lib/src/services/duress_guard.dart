import 'package:eel_kit_sdk/eel_kit_sdk.dart';

/// The covert duress PIN. If a worker is forced to check in, entering this PIN
/// instead of their own silently raises a `KitDuressAlert` via the offline
/// queue while the UI continues exactly as if nothing happened.
const String kDuressPin = '000000';

/// Wraps [EelDuressHook] for the personnel app. Callers must keep the true and
/// false branches visually identical to avoid side-channel leakage.
class DuressGuard {
  DuressGuard({EelOfflineQueue? queue, String duressPin = kDuressPin})
      : _hook = EelDuressHook(
          queue: queue ?? EelOfflineQueue(storage: InMemoryOutboxStorage()),
          matcher: (input) => input.trim() == duressPin,
          deviceIdProvider: () => 'personnel-device',
        );

  final EelDuressHook _hook;

  /// Returns true when [pin] is the duress PIN (and a covert alert was
  /// enqueued). The result must not change what the user sees.
  Future<bool> check(String pin) => _hook.check(pin);
}
