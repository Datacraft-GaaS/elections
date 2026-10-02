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

import 'dart:convert';

import 'package:eel_kit_sdk/eel_kit_sdk.dart';

/// Signs an attendance payload — Ed25519 over canonical JSON — with the kit's
/// device signing key, matching the bundle-security trust model. The signature
/// is sent alongside the request so the dispatch-api can later verify the
/// event came from the enrolled device.
Future<String> signAttendancePayload(Map<String, Object?> payload) async {
  final key = await EelSigningKey.loadOrCreate();
  final canonical = eelCanonicalJson(payload);
  return key.sign(utf8.encode(canonical));
}
