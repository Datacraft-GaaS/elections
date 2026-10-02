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

// Real attestation via kit-sdk (ADR-0021: hard-fail MDM attestation before any auth).
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

class AttestationService {
  AttestationService({required EelAttestationClient client}) : _client = client;

  final EelAttestationClient _client;

  /// Attest the device and return whether the verdict is acceptable.
  ///
  /// The client is a required parameter: EelAttestationClient is abstract, so
  /// there is no instance to fall back to, and quietly substituting the
  /// software-tier stub would turn a hardware check into a rubber stamp.
  Future<AttestationResult> attest() async {
    final verdict = await _client.attest();
    if (_isAcceptable(verdict)) {
      return const AttestationResult(passed: true);
    }
    return AttestationResult(
      passed: false,
      failureReason: _reason(verdict),
    );
  }

  /// Hardware-backed and unexpired, or it does not count.
  ///
  /// A softwareFallback tier means no hardware attestation happened, so the
  /// verdict says nothing about the device's integrity. An expired verdict
  /// says nothing about its present state. Either way this is a rejection.
  static bool _isAcceptable(AttestationVerdict v) =>
      v.tier != AttestationTier.softwareFallback &&
      v.expiresAt.isAfter(DateTime.now());

  static String _reason(AttestationVerdict v) {
    if (v.tier == AttestationTier.softwareFallback) {
      return 'Attestation is software-tier only, not hardware backed';
    }
    return 'Attestation expired at ${v.expiresAt.toIso8601String()}';
  }
}

class AttestationResult {
  final bool passed;
  final String? token;
  final String? failureReason;

  const AttestationResult({
    required this.passed,
    this.token,
    this.failureReason,
  });
}
