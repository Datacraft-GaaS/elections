/// RTS Kit driver (M08). Per plan.md M3: SMS fallback real status responses.
/// Hardware: GSM modem for SMS.

class SmsStatus {
  final bool sent;
  final String? statusCode;
  final String? carrierMessageId;
  final DateTime? sentAt;
  final String? error;

  const SmsStatus({
    this.sent = false, this.statusCode, this.carrierMessageId, this.sentAt, this.error,
  });

  factory SmsStatus.invalid(String reason) => SmsStatus(error: reason);
}

/// RTS driver — GSM-gated per plan.md M3.
class RtsDriver {
  /// Send SMS status fallback when no data connection
  Future<SmsStatus> sendSmsFallback(String phone, String statusCode) async {
    throw UnimplementedError(
      'RtsDriver.sendSmsFallback requires GSM modem. '
      'Per plan.md M3: GSM hardware + SIM card required.',
    );
  }

  /// Check if SMS path is available
  Future<bool> isSmsAvailable() async {
    throw UnimplementedError(
      'RtsDriver.isSmsAvailable requires GSM modem. '
      'Per plan.md M3: GSM hardware required before use.',
    );
  }

  /// Self-check hardware
  Future<bool> selfCheck() async {
    throw UnimplementedError(
      'RtsDriver.selfCheck requires GSM self-test routine. '
      'Per plan.md M3: GSM hardware required before use.',
    );
  }
}
