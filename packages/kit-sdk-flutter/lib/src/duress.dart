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
