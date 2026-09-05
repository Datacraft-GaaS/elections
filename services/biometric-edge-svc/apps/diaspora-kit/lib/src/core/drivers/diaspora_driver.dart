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
  Future<HsmKeyResult> generateRealKey() async {
    throw UnimplementedError(
      'DiasporaDriver.generateRealKey requires HSM device. '
      'Per plan.md M3: HSM hardware + PKCS#11 module required.',
    );
  }

  Future<DiasporaBatchUpload> queueDeferredBatch() async {
    throw UnimplementedError(
      'DiasporaDriver.queueDeferredBatch requires diaspora batch upload service. '
      'Per plan.md M3: diaspora back-end integration required.',
    );
  }

  Future<bool> selfCheck() async {
    throw UnimplementedError(
      'DiasporaDriver.selfCheck requires HSM self-test routine. '
      'Per plan.md M3: HSM hardware required before use.',
    );
  }
}
