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
