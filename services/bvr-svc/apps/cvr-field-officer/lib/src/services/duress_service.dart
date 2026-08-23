/// Duress alert service.
///
/// Long-press on the capture button triggers a covert KitDuressAlert event
/// through the kit-sdk duress hook. The event is queued for ECC dispatch and
/// no visible feedback distinguishes it from normal operation (DPA: a visible
/// response would reveal the officer's situation to a coercer).
///
/// Durability currently matches the rest of the app (in-memory outbox until
/// Isar codegen lands); alerts are flushed by the sync engine on reconnect.
library;

import 'package:eel_kit_sdk/eel_kit_sdk.dart';

import 'app_identity.dart';

String _deviceIdOrDefault() {
  try {
    return AppIdentity.instance.officerId;
  } on StateError {
    return 'device-unattested';
  }
}

class DuressService {
  DuressService._();

  static final DuressService instance = DuressService._();

  /// Sentinel input that the explicit long-press path feeds to the hook.
  static const String _longPressToken = 'capture_long_press';

  late final EelDuressHook _hook = EelDuressHook(
    queue: EelOfflineQueue(storage: InMemoryOutboxStorage()),
    matcher: (input) => input == _longPressToken,
    deviceIdProvider: _deviceIdOrDefault,
  );

  /// Emit a silent duress alert to ECC dispatch.
  ///
  /// Returns true when the alert event was queued. Never surfaces UI state:
  /// the caller must keep the duress and non-duress branches visually
  /// identical to avoid side-channel leakage.
  Future<bool> triggerLongPressAlert() => _hook.check(_longPressToken);
}
