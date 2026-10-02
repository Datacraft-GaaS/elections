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

/// RTS Kit driver (M08). Per plan.md M3: SMS fallback real status responses.
/// Hardware: GSM modem for SMS.
library;

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
  Future<SmsStatus> sendSmsFallback(String phone, String statusCode) async =>
      SmsStatus.invalid('gsm_modem_not_procured — hardware-gated per plan.md M3');

  Future<bool> isSmsAvailable() async => false; // no GSM modem attached

  Future<bool> selfCheck() async => false; // no GSM modem to test
}
