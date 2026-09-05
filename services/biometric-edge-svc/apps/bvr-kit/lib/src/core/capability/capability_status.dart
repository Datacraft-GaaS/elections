/// Hardware capability detection — M3 gate check (no hardware required to run).
/// Per plan.md M3: all 4 kits must report self-check status before feature screens open.
/// Returns structured status that feature screens can render as banners/blocks.

class KitCapabilityStatus {
  final bool fingerprintAvailable;
  final bool cameraAvailable;
  final bool signatureAvailable;
  final bool ntsAvailable;
  final bool gnssAvailable;
  final bool hsmAvailable;
  final bool smsAvailable;
  final String? reason; // explanation for unavailable items

  const KitCapabilityStatus({
    this.fingerprintAvailable = false,
    this.cameraAvailable = false,
    this.signatureAvailable = false,
    this.ntsAvailable = false,
    this.gnssAvailable = false,
    this.hsmAvailable = false,
    this.smsAvailable = false,
    this.reason,
  });

  bool get isFullyCapable =>
      fingerprintAvailable &&
      cameraAvailable &&
      signatureAvailable;

  bool get isMinimal => fingerprintAvailable || cameraAvailable || gnssAvailable;

  factory KitCapabilityStatus.fromSelfCheck(Map<String, dynamic> map) =>
      KitCapabilityStatus(
        fingerprintAvailable: map['fingerprint'] == true,
        cameraAvailable: map['camera'] == true,
        signatureAvailable: map['signature'] == true,
        ntsAvailable: map['nts'] == true,
        gnssAvailable: map['gnss'] == true,
        hsmAvailable: map['hsm'] == true,
        smsAvailable: map['sms'] == true,
        reason: map['reason'] as String?,
      );

  /// Graceful message for feature-screen banners when hardware unavailable.
  String get featureBannerMessage => isFullyCapable
      ? 'All hardware available'
      : 'Hardware required: ${!fingerprintAvailable ? "FAP60 fingerprint " : ""}${!cameraAvailable ? "document camera " : ""}${!signatureAvailable ? "signature pad" : ""}. Per plan.md M3: procure devices before feature use.';
}
