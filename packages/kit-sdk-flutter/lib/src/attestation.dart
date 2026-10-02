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

import 'package:flutter/foundation.dart';

enum AttestationTier { playIntegrity, appAttest, tpm, softwareFallback }

class AttestationVerdict {
  const AttestationVerdict({
    required this.deviceId,
    required this.tier,
    required this.expiresAt,
  });

  final String deviceId;
  final AttestationTier tier;
  final DateTime expiresAt;
}

abstract class EelAttestationClient {
  Future<AttestationVerdict> attest();
}

/// Software-tier stub for development and testing. Real backends (Play Integrity,
/// App Attest, TPM) are wired at kit runtime.
class EelAttestationClientStub implements EelAttestationClient {
  @override
  Future<AttestationVerdict> attest() async {
    return AttestationVerdict(
      deviceId: 'stub-device-${defaultTargetPlatform.name}',
      tier: AttestationTier.softwareFallback,
      expiresAt: DateTime.now().add(const Duration(hours: 1)),
    );
  }
}
