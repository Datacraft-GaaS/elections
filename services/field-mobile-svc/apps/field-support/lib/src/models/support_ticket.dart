enum TicketSeverity { sev1, sev2, sev3 }
enum TicketStatus { open, inProgress, escalated, resolved, closed }

class SupportTicket {
  final String id;
  final String stationCode;
  final String kitSerial;
  final String reporterRole;
  final TicketSeverity severity;
  final String title;
  final String description;
  final TicketStatus status;
  final String? assignedEngineerId;
  final DateTime slaTargetAt;
  final DateTime? resolvedAt;
  final int escalationLevel;

  const SupportTicket({
    required this.id,
    required this.stationCode,
    required this.kitSerial,
    required this.reporterRole,
    required this.severity,
    required this.title,
    required this.description,
    required this.status,
    this.assignedEngineerId,
    required this.slaTargetAt,
    this.resolvedAt,
    this.escalationLevel = 0,
  });
}
