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

/// Device identity for the CVR field-officer app.
///
/// Wraps the kit-sdk attestation module: the device attests once per session
/// and the resulting verdict supplies both the officer identity recorded on
/// captures and the SPIFFE-style workload id sent in API headers.
library;

import 'package:eel_kit_sdk/eel_kit_sdk.dart';

class AppIdentity {
  AppIdentity._();

  static final AppIdentity instance = AppIdentity._();

  final EelAttestationClient _client = EelAttestationClientStub();

  AttestationVerdict? _verdict;

  /// Attest the device (cached until the verdict expires).
  Future<AttestationVerdict> ensureAttested() async {
    final current = _verdict;
    if (current != null && current.expiresAt.isAfter(DateTime.now())) {
      return current;
    }
    return _verdict = await _client.attest();
  }

  /// Kit-bound officer identity used as `officerId` on captures.
  ///
  /// Officer enrolment maps a provisioned badge to this device-attested id;
  /// the raw device id is what the backend can verify from the attestation.
  String get officerId {
    final v = _requireVerdict();
    return v.deviceId;
  }

  /// Workload identity for the X-Spiffe-Svid header, derived from the
  /// attested device id using the platform trust-domain conventions
  /// (`spiffe://eel.local/ns/<ns>/sa/<name>`).
  String get spiffeId {
    final v = _requireVerdict();
    return 'spiffe://eel.local/ns/cvr/sa/kit-${v.deviceId}';
  }

  AttestationVerdict _requireVerdict() {
    final v = _verdict;
    if (v == null) {
      throw StateError(
        'Device has not attested yet; call ensureAttested() first',
      );
    }
    return v;
  }
}
