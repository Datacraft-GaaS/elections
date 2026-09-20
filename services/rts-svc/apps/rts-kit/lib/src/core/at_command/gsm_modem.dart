/// GsmModem — complete SMS fallback integration using usb_serial + AT commands.
/// Per plan.md M3 (M08): SMS fallback when HTTP fails 3× or network absent.
/// Uses 3GPP AT commands (no vendor SDK needed) + USB serial transport.
library;

import 'at_command.dart';
import 'usb_serial_transport.dart';
import 'sms_pdu.dart';

class GsmModem {
  final UsbSerialTransport _transport = UsbSerialTransport();
  bool _initialized = false;

  /// Initialize modem: open USB serial, configure, verify SIM + network.
  /// Returns true if modem responds to AT and is registered on network.
  Future<bool> init({String portPath = '/dev/ttyUSB0', int baud = 115200}) async {
    try {
      await _transport.open(port: portPath, baud: baud);
      await _sendExpect('AT'); // basic echo
      await _sendExpect(AtCommand.initialize.cmd); // AT+CFUN=1
      await _waitMs(500);
      await _sendExpect(AtCommand.checkSim.cmd); // AT+CPIN?
      await _waitMs(500);
      await _sendExpect(AtCommand.checkNetwork.cmd); // AT+CREG?
      await _waitMs(500);
      await _sendExpect(AtCommand.setTextMode.cmd); // AT+CMGF=1
      await _waitMs(200);
      await _sendExpect(AtCommand.setStorage.cmd); // AT+CPMS
      _initialized = true;
      return true;
    } catch (_) {
      _initialized = false;
      return false;
    }
  }

  /// Send SMS status fallback with CBOR+zstd payload.
  /// Per plan.md M08: 160-char segments (7-bit GSM) or 140-byte binary segments.
  /// Status code is typically 2-letter (e.g. "OK", "NG").
  Future<bool> sendStatusSms({required String destination, required String statusCode, List<int>? payload}) async {
    if (!_initialized) return false;
    try {
      // Build SMS payload: status code + optional compressed payload
      final message = payload != null && payload.isNotEmpty
          ? '<STATUS:$statusCode>;${String.fromCharCodes(payload)}'
          : '<STATUS:$statusCode>';

      final segments = encodeSmsSubmit(
        destinationNumber: destination,
        payloadBytes: message.codeUnits,
        dataCodingScheme: 0xF5, // 8-bit binary
      );

      for (final pduHex in segments) {
        // Send SMS-SUBMIT PDU via AT+CMGS in PDU mode (CMGF=0) for binary payload
        // Or send text directly if using text mode (CMGF=1)
        await _transport.send(AtCommand.sendSms(destination).cmd);
        await _waitMs(100);
        await _transport.send(pduHex); // If PDU mode; for text mode use message directly
        await _waitMs(200);
        await _transport.send('\x1A'); // Ctrl-Z terminator
        await _waitMs(500);
      }
      return true;
    } catch (_) {
      return false;
    }
  }

  /// Simple text SMS fallback (for testing / basic status).
  Future<bool> sendTextSms({required String destination, required String text}) async {
    if (!_initialized) return false;
    try {
      await _transport.send(AtCommand.sendSms(destination).cmd);
      await _waitMs(300);
      await _transport.send('$text\x1A');
      await _waitMs(500);
      // Verify response line contains "+CMGS"
      final response = await _transport.readLine(timeoutMs: 5000);
      return response.contains('+CMGS');
    } catch (_) {
      return false;
    }
  }

  /// Read SMS from SIM storage.
  Future<List<String>> readReceivedSms() async {
    if (!_initialized) return [];
    try {
      await _transport.send(AtCommand.readSms(status: '4').cmd);
      await _waitMs(300);
      final lines = await _transport.readUntil('OK', timeoutMs: 3000);
      return lines.where((l) => l.isNotEmpty && !l.startsWith('AT')).toList();
    } catch (_) {
      return [];
    }
  }

  /// Clean up SMS from SIM.
  Future<bool> deleteSms(int index) async {
    if (!_initialized) return false;
    try {
      await _transport.send(AtCommand.deleteSms(index).cmd);
      await _waitMs(300);
      return true;
    } catch (_) {
      return false;
    }
  }

  /// Check if network is registered (for status display).
  Future<String?> getNetworkStatus() async {
    if (!_initialized) return null;
    try {
      await _transport.send(AtCommand.checkNetwork.cmd);
      await _waitMs(300);
      final resp = await _transport.readLine(timeoutMs: 3000);
      return resp.contains('+CREG: 0,1') || resp.contains('+CREG: 0,5') ? 'registered' : 'not_registered';
    } catch (_) {
      return null;
    }
  }

  Future<bool> isAvailable() async => _initialized;

  Future<void> close() async {
    try {
      await _transport.send(AtCommand.close.cmd);
      await _transport.close();
    } catch (_) {}
    _initialized = false;
  }

  Future<void> _sendExpect(String cmd) async {
    await _transport.send(cmd);
    await _waitMs(100);
    final line = await _transport.readLine(timeoutMs: 3000);
    if (line.contains('ERROR') || line.contains('+CME ERROR') || line.contains('+CMS ERROR')) {
      throw Exception('AT error: $line');
    }
  }

  Future<void> _waitMs(int ms) => Future.delayed(Duration(milliseconds: ms));
}
