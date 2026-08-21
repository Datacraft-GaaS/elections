class DiagnosticAction {
  final String id;
  final String ticketId;
  final String kitSerial;
  final String actionType;
  final String engineerId;
  final DateTime signedAt;
  final String result;

  const DiagnosticAction({
    required this.id,
    required this.ticketId,
    required this.kitSerial,
    required this.actionType,
    required this.engineerId,
    required this.signedAt,
    required this.result,
  });
}
