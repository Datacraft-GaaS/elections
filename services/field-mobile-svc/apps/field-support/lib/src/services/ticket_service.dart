import 'package:dio/dio.dart';

class TicketService {
  final Dio _dio;

  TicketService({Dio? dio}) : _dio = dio ?? Dio();

  Future<List<Map<String, dynamic>>> fetchOpenTickets(String engineerId) async {
    final response = await _dio.get('/api/tickets', queryParameters: {
      'assignedTo': engineerId,
      'status': 'open',
    });
    return List<Map<String, dynamic>>.from(response.data as List);
  }

  Future<void> escalateTicket(String ticketId, int toLevel) async {
    await _dio.post('/api/tickets/$ticketId/escalate', data: {'level': toLevel});
  }
}
