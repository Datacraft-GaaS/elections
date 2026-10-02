// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// OTP service implementation (G01)
/// Integrates with integrations/telco-sms/ adapter for actual SMS/voice OTP delivery.
/// Calls: sendOtp(phoneNumber) -> SMS/voice via telco-sms adapter, logs an audit entry.
///
/// Every call writes a 5-axis audit entry (where / when / who / why / how) via
/// [AuditService], per AGENTS.md and ADR-0021.

import 'dart:async';
import 'dart:convert';

import 'package:crypto/crypto.dart';
import 'package:http/http.dart' as http;

import 'package:eel_core_flutter/eel_core_flutter.dart';

class OtpService {
  OtpService({
    required this.audit,
    this.appName = 'eel',
    this.baseUrl = _defaultBaseUrl,
    this.timeout = _defaultTimeout,
  });

  static const String _defaultBaseUrl = 'https://api.telco-sms.internal/v1/otp';
  static const Duration _defaultTimeout = Duration(seconds: 10);

  final AuditService audit;
  final String appName;
  final String baseUrl;
  final Duration timeout;

  /// Send OTP to phone number using the telco-sms adapter.
  /// An audit entry is emitted on every outcome: sent, failed, or exception.
  Future<OtpResponse> sendOtp(String phoneNumber) async {
    final payload = jsonEncode(<String, dynamic>{'phoneNumber': phoneNumber});

    try {
      final response = await http
          .post(
            Uri.parse('$baseUrl/send'),
            headers: {'Content-Type': 'application/json'},
            body: payload,
          )
          .timeout(timeout);

      if (response.statusCode == 200) {
        final decoded = jsonDecode(response.body) as Map<String, dynamic>;
        await _log(
          action: 'otp_sent',
          target: phoneNumber,
          reason: 'user requested an OTP',
          method: 'telco-sms',
          proof: response.body,
        );
        return OtpResponse.ok(decoded['token']?.toString() ?? '');
      }

      await _log(
        action: 'otp_send_failed',
        target: phoneNumber,
        reason: 'adapter returned HTTP ${response.statusCode}',
        method: 'telco-sms',
        proof: '${response.statusCode}${response.body}',
      );
      return OtpResponse.err('HTTP ${response.statusCode}');
    } on Exception catch (e) {
      await _log(
        action: 'otp_send_exception',
        target: phoneNumber,
        reason: e.toString(),
        method: 'telco-sms',
        proof: e.toString(),
      );
      return OtpResponse.err(e.toString());
    }
  }

  Future<void> _log({
    required String action,
    required String target,
    required String reason,
    required String method,
    required String proof,
  }) {
    return audit.log(
      actorId: target,
      actorRole: 'CLERK',
      action: action,
      target: target,
      reason: reason,
      // The adapter call is server-side; the station's location is not known to
      // this layer. Left explicit rather than invented.
      whereGps: 'unknown',
      howMethod: method,
      howProofHash: sha256.convert(utf8.encode(proof)).toString(),
      // This layer holds no signing key; the adapter signs its own receipt.
      howSignature: 'unsigned',
      isOffline: false,
    );
  }
}

/// OTP response model
class OtpResponse {
  final bool success;
  final String? token;
  final String? error;

  OtpResponse._(this.success, {this.token, this.error});

  /// Named ok/err rather than success/failure: the class already has an
  /// instance field `success`, and Dart forbids a static and an instance
  /// member of the same name on one class.
  static OtpResponse ok(String token) => OtpResponse._(true, token: token);
  static OtpResponse err(String error) => OtpResponse._(false, error: error);
}
