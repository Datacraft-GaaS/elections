library;

import 'dart:async';
import 'dart:typed_data';
import 'package:usb_serial/usb_serial.dart';
import 'at_command.dart';

class UsbSerialTransport implements AtSerialTransport {
  UsbPort? _port;
  bool _open = false;

  @override
  Future<void> open({String port = '/dev/ttyUSB0', int baud = 115200}) async {
    final devices = await UsbSerial.listDevices();
    if (devices.isEmpty) {
      throw StateError('No USB serial devices found (modem not connected?)');
    }
    final device = devices.firstWhere(
      (d) =>
          d.productName?.toLowerCase().contains('sim') == true ||
          d.productName?.toLowerCase().contains('quectel') == true ||
          d.productName?.toLowerCase().contains('telit') == true,
      orElse: () => devices.first,
    );
    _port = await device.createPort();
    if (_port == null) {
      throw StateError(
          'Failed to open USB port for device ${device.productName}');
    }
    await _port!.open();
    await _port!.setDTR(true);
    await _port!.setRTS(true);
    await _port!.setPortParameters(
        baud, UsbPort.DATABITS_8, UsbPort.STOPBITS_1, UsbPort.PARITY_NONE);
    _open = true;
  }

  @override
  Future<void> send(String data) async {
    if (!_open || _port == null) throw StateError('Serial port not open');
    await _port!.write(Uint8List.fromList('$data\r\n'.codeUnits));
  }

  @override
  Future<String> readLine({int timeoutMs = 5000}) async {
    if (!_open || _port == null) throw StateError('Serial port not open');
    final completer = Completer<String>();
    final buffer = StringBuffer();
    late final StreamSubscription<Uint8List> sub;
    sub = _port!.inputStream!.listen((Uint8List event) {
      final s = String.fromCharCodes(event);
      buffer.write(s);
      final line = buffer.toString();
      if (line.contains('\r\n') || line.contains('\n')) {
        final idx = line.indexOf('\n');
        final result = line.substring(0, idx); // exclude newline
        buffer.clear();
        if (line.length > idx + 1) {
          buffer.write(line.substring(idx + 1));
        }
        sub.cancel();
        completer.complete(result.trim());
      }
    }, onError: (e) {
      sub.cancel();
      completer.completeError(e);
    });
    await Future.delayed(Duration(milliseconds: timeoutMs));
    if (!completer.isCompleted) {
      sub.cancel();
      completer.complete(buffer.toString().trim());
    }
    return await completer.future;
  }

  @override
  Future<List<String>> readUntil(String terminator,
      {int timeoutMs = 5000}) async {
    if (!_open || _port == null) throw StateError('Serial port not open');
    final lines = <String>[];
    final buffer = StringBuffer();
    const chunkSize = 1024;
    final deadline = DateTime.now().add(Duration(milliseconds: timeoutMs));
    while (DateTime.now().isBefore(deadline)) {
      try {
        final chunk = await _port!.read(chunkSize);
        if (chunk != null && chunk.isNotEmpty) {
          buffer.write(String.fromCharCodes(chunk));
          var s = buffer.toString();
          while (s.contains(terminator)) {
            final idx = s.indexOf(terminator);
            lines.add(s.substring(0, idx).trim());
            s = s.substring(idx + terminator.length);
          }
          buffer.clear();
          buffer.write(s);
          if (lines.isNotEmpty && s.isEmpty) break; // Got full block
        } else {
          await Future.delayed(const Duration(milliseconds: 50));
        }
      } catch (_) {
        await Future.delayed(const Duration(milliseconds: 50));
      }
    }
    if (buffer.isNotEmpty) lines.add(buffer.toString().trim());
    return lines;
  }

  @override
  Future<void> close() async {
    _open = false;
    await _port?.close();
    _port = null;
  }

  @override
  bool get isOpen => _open;
}
