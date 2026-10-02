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

import 'package:flutter/services.dart';

/// Channel names served by the eel_drivers native plugin. These match the
/// contracts in kit-sdk (`eel/iris`, `eel/fingerprint`) and the factory
/// test harness (`eel.bvi/*`, `eel/modem/sms`).
abstract final class EelDriverChannels {
  static const iris = 'eel.bvi/iris';
  static const irisAlias = 'eel/iris';
  static const fingerprint = 'eel/fingerprint';
  static const printer = 'eel/bvi/printer';
  static const sms = 'eel/modem/sms';
  static const scanner = 'eel/bvi/document_scanner';
  static const camera = 'eel/bvi/camera';

  static const all = [
    iris,
    irisAlias,
    fingerprint,
    printer,
    sms,
    scanner,
    camera,
  ];
}

/// Typed view over every channel's `probeDetails` method:
/// `{present: bool, reason: String, devices: [...]}`.
/// `present == false` means "no such hardware attached" (harness SKIP),
/// as opposed to a missing plugin (harness SKIP with a different message)
/// or a failed operation on present hardware (harness FAIL).
abstract final class EelDriverProbe {
  static Future<Map<String, Object?>> probe(String channel) async {
    const codec = StandardMethodCodec();
    final result = await MethodChannel(channel, codec)
        .invokeMethod<Map<Object?, Object?>>('probeDetails');
    return (result ?? {}).cast<String, Object?>();
  }

  static Future<bool> present(String channel) async =>
      (await probe(channel))['present'] == true;
}