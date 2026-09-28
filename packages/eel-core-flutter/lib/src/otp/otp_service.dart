/// OTP service implementation (G01)
/// Integrates with integrations/telco-sms/ adapter for actual SMS/voice OTP delivery.
/// Calls: sendOtp(phone_number) → SMS/voice via telco-sms adapter, logs audit event.
/// M5 auditId: otp-{timestamp}

import 'dart:async';
import 'dart:convert';
import 'package:http/http.dart' as http;
import 'package:uuid/uuid.dart';
import 'package:eel_core_flutter/src/audit/audit_service.dart';

class OtpService {
  static const String _baseUrl = 'https://api.telco-sms.internal/v1/otp';
  static const Duration _timeout = Duration(seconds: 10);

  /// Send OTP to phone number using telco-sms adapter.
  /// M5 audit event emitted on every call.
  static Future<OtpResponse> sendOtp(String phoneNumber) async {
    final String auditId = 'otp-${Uuid().v7()}';
    final Map<String, dynamic> request = {'phoneNumber': phoneNumber};
    final String payload = jsonEncode(request);

    try {
      final response = await http.post(
        Uri.parse('$_baseUrl/send'),
        headers: {'Content-Type': 'application/json'},
        body: payload,
      ).timeout(_timeout);

      if (response.statusCode == 200) {
        final Map<String, dynamic> responseJson = jsonDecode(response.body);
        final auditEvent = AuditEvent(
          auditId: auditId,
          timestamp: DateTime.now(),
          eventType: 'otp_sent',
          userId: phoneNumber,
          details: responseJson,
          proofHash: sha256(response.body).toString(),
        );
        await AuditService.instance.emitEvent(auditEvent);
        return OtpResponse.success(responseJson['token']?.toString());
      } else {
        final auditEvent = AuditEvent(
          auditId: auditId,
          timestamp: DateTime.now(),
          eventType: 'otp_send_failed',
          userId: phoneNumber,
          details: {'error': response.statusCode, 'body': response.body},
          proofHash: sha256('${response.statusCode}${response.body}').toString(),
        );
        await AuditService.instance.emitEvent(auditEvent);
        return OtpResponse.failure('HTTP ${response.statusCode}');
      }
    } on Exception catch (e) {
      final auditEvent = AuditEvent(
        auditId: auditId,
        timestamp: DateTime.now(),
        eventType: 'otp_send_exception',
        userId: phoneNumber,
        details: {'exception': e.toString()},
        proofHash: sha256(e.toString()).toString(),
      );
      await AuditService.instance.emitEvent(auditEvent);
      return OtpResponse.failure(e.toString());
    }
  }
}

/// OTP response model
class OtpResponse {
  final bool success;
  final String? token;
  final String? error;

  OtpResponse._(this.success, {this.token, this.error});

  static OtpResponse success(String token) => OtpResponse._(true, token: token);
  static OtpResponse failure(String error) => OtpResponse._(false, error: error);
}