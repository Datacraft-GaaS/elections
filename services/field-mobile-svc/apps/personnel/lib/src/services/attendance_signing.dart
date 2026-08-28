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
