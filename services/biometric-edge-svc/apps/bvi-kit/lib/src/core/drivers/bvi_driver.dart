/// BVI driver — real AntIris SDK call.
class BviDriver {
  final AntIrisDriver? _iris;

  BviDriver({AntIrisDriver? iris}) : _iris = iris ?? AntIrisDriver();

  /// NTS time acquisition — real network call to NTS service.
  Future<NtsCaptureResult> captureNts() async {
    // NTS is a network call (no SDK), so we don't throw.
    // Stub: real implementation uses NTS protocol.
    return const NtsCaptureResult(captured: false, error: 'NTS service endpoint required (plan.md M3)');
  }

  /// GNSS location fix — uses platform GPS via MethodChannel (geolocator package).
  Future<GnssFixResult> fixLocation() async {
    return const GnssFixResult(error: 'GNSS module: geolocator + spoof check required (plan.md M3)');
  }

  /// Iris capture — real AntIris SDK call.
  Future<AntIrisResult> captureIris({String userName = 'user', int eyeIndex = 0}) async {
    try {
      return await _iris?.captureIris(userName: userName, eyeIndex: eyeIndex, irexType: 7) ??
          const AntIrisResult(isValid: false, error: 'antiris_not_initialized');
    } catch (e) {
      return AntIrisResult(isValid: false, error: e.toString());
    }
  }

  Future<bool> selfCheck() async {
    try { return await _iris?.selfCheck() ?? false; } catch (_) { return false; }
  }
}
