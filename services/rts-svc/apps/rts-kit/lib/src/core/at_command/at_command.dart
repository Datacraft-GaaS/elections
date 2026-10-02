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

/// Generic GSM AT command library — portable across all GSM modems.
/// Standard: 3GPP TS 27.007 / TS 07.05.
/// No vendor SDK required. Uses platform serial (USB ACM / ttyUSB) via MethodChannel.
/// This library defines the command vocabulary and response parser; hardware access
/// is delegated to the native bridge (SerialPortMethodChannel).
///
/// M08 SMS fallback (plan.md M3): uses AT+CMGF, AT+CMGS, AT+CPMS, SMS-Submit PDU.
library;

/// AT command definition — command string + expected response pattern.
class AtCommand {
  final String cmd;          /// e.g. "AT+CMGF=1"
  final int timeoutMs;       /// Default 5000
  final List<String>? okPatterns;   /// e.g. ["OK"]
  final List<String>? errorPatterns; /// e.g. ["ERROR", "+CME ERROR", "+CMS ERROR"]

  const AtCommand({
    required this.cmd,
    this.timeoutMs = 5000,
    this.okPatterns = const ['OK'],
    this.errorPatterns = const ['ERROR', '+CME ERROR', '+CMS ERROR'],
  });

  // --- SMS commands ---
  static const setTextMode = AtCommand(cmd: 'AT+CMGF=1', timeoutMs: 3000);
  static const setPduMode = AtCommand(cmd: 'AT+CMGF=0', timeoutMs: 3000);
  static const setGsmCharset = AtCommand(cmd: 'AT+CSCS="GSM"', timeoutMs: 3000);
  static const setStorage = AtCommand(cmd: 'AT+CPMS="SM","SM","SM"', timeoutMs: 3000);
  static const checkStorage = AtCommand(cmd: 'AT+CPMS?', timeoutMs: 3000);
  static const initialize = AtCommand(cmd: 'AT+CFUN=1', timeoutMs: 5000);
  static const checkSim = AtCommand(cmd: 'AT+CPIN?', timeoutMs: 3000);
  static const checkNetwork = AtCommand(cmd: 'AT+CREG?', timeoutMs: 3000);
  static const close = AtCommand(cmd: 'AT+CFUN=0', timeoutMs: 3000);

  /// Build SMS send command with recipient number.
  /// AT+CMGS="+1234567890" sends to number; requires terminating \032 (Ctrl-Z).
  static AtCommand sendSms(String number, {int timeoutMs = 10000}) =>
      AtCommand(cmd: 'AT+CMGS="$number"', timeoutMs: timeoutMs);

  /// Read SMS from storage (status 4 = unread, 1 = unread, 0 = read, 3 = sent).
  static AtCommand readSms({String status = '4'}) =>
      AtCommand(cmd: 'AT+CMGL=$status', timeoutMs: 3000);

  /// Delete SMS by index.
  static AtCommand deleteSms(int index) =>
      AtCommand(cmd: 'AT+CMGD=$index', timeoutMs: 3000);
}

/// AT response — parsed from raw line(s) returned by modem.
class AtResponse {
  final String raw;       /// Full multi-line response (includes OK/ERROR)
  final bool ok;          /// Contains OK (not ERROR)
  final bool isCmeError;  /// Contains +CME ERROR: <code>,<msg>
  final bool isCmsError;  /// Contains +CMS ERROR: <code>,<msg>
  final String? cmeCode;  /// Numeric error code if CME
  final String? cmeMsg;   /// Human message if CME
  final String? data;     /// Data payload (e.g. SMS text, signal strength)

  const AtResponse({
    required this.raw,
    required this.ok,
    this.isCmeError = false,
    this.isCmsError = false,
    this.cmeCode,
    this.cmeMsg,
    this.data,
  });

  /// Parse raw line from modem.
  factory AtResponse.parse(String line) {
    final trimmed = line.trim();
    final hasOk = trimmed.contains('OK');
    final hasErr = trimmed.contains('ERROR') || trimmed.contains('+CME ERROR') || trimmed.contains('+CMS ERROR');
    final cmeMatch = RegExp(r'\+CME ERROR:\s*([0-9]+),([^\n]+)').firstMatch(trimmed);
    final cmsMatch = RegExp(r'\+CMS ERROR:\s*([0-9]+),([^\n]+)').firstMatch(trimmed);
    return AtResponse(
      raw: trimmed,
      ok: hasOk && !hasErr,
      isCmeError: cmeMatch != null,
      isCmsError: cmsMatch != null,
      cmeCode: cmeMatch?.group(1),
      cmeMsg: cmeMatch?.group(2)?.trim(),
      data: hasOk && !trimmed.startsWith('AT+') ? trimmed : null, // heuristic: data lines don't start with AT+
    );
  }
}

/// Serial transport abstraction — implemented by native plugin or POSIX serial.
abstract class AtSerialTransport {
  Future<void> open({String port = '/dev/ttyUSB0', int baud = 115200});
  Future<void> send(String data); // writes with \r\n terminator
  Future<String> readLine({int timeoutMs = 5000});
  Future<List<String>> readUntil(String terminator, {int timeoutMs = 5000});
  Future<void> close();
  bool get isOpen;
}
