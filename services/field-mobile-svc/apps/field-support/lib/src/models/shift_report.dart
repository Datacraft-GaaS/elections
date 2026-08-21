class ShiftReport {
  final String id;
  final String engineerId;
  final String clusterId;
  final DateTime shiftStart;
  final DateTime shiftEnd;
  final int ticketsHandled;
  final double slaCompliancePct;
  final int escalations;
  final int sparesConsumed;
  final String engineerSignature;

  const ShiftReport({
    required this.id,
    required this.engineerId,
    required this.clusterId,
    required this.shiftStart,
    required this.shiftEnd,
    required this.ticketsHandled,
    required this.slaCompliancePct,
    required this.escalations,
    required this.sparesConsumed,
    required this.engineerSignature,
  });
}
