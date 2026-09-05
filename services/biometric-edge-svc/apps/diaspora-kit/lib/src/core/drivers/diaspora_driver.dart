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
