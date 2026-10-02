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

/// Diaspora Kit hardware driver (M24). Per plan.md M3: Real HSM key + deferred batch upload.
/// Hardware: HSM device (for real key, not fabricated).

class HsmKeyResult {
  final bool generated;
  final String? keyFingerprint;
  final String? publicKey;
  final String? error;

  const HsmKeyResult({
    this.generated = false, this.keyFingerprint, this.publicKey, this.error,
  });

  factory HsmKeyResult.invalid(String reason) => HsmKeyResult(error: reason);
}

class DiasporaBatchUpload {
  final bool queued;
  final String? batchId;
  final String? error;

  const DiasporaBatchUpload({
    this.queued = false, this.batchId, this.error,
  });

  factory DiasporaBatchUpload.invalid(String reason) => DiasporaBatchUpload(error: reason);
}

/// Diaspora driver — HSM-gated per plan.md M3.
class DiasporaDriver {
  Future<HsmKeyResult> generateRealKey() async => HsmKeyResult.invalid('HSM device not present — hardware-gated per plan.md M3');

  Future<DiasporaBatchUpload> queueDeferredBatch() async => DiasporaBatchUpload.invalid('Diaspora batch upload service unavailable — back-end integration not present');

  Future<bool> selfCheck() async => false; // HSM self-test unavailable — hardware not procured
}
