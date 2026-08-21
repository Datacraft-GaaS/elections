enum ClaimStatus { draft, submitted, approved, rejected }

class WarrantyClaim {
  final String id;
  final String kitSerial;
  final String oemCode;
  final String faultDescription;
  final ClaimStatus claimStatus;
  final String engineerId;
  final DateTime submittedAt;

  const WarrantyClaim({
    required this.id,
    required this.kitSerial,
    required this.oemCode,
    required this.faultDescription,
    required this.claimStatus,
    required this.engineerId,
    required this.submittedAt,
  });
}
