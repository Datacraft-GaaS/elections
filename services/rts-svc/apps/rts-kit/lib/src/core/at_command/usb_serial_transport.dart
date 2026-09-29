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
    // UsbDevice.deviceName is non-nullable in usb_serial, so there is no
    // missing-name case to guard here.
    _port = UsbPort(device.deviceName);
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
      await sub.cancel();
      completer.complete(buffer.toString().trim());
    }
    return await completer.future;
  }

  @override
  Future<List<String>> readUntil(String terminator,
      {int timeoutMs = 5000}) async {
    if (!_open || _port == null) throw StateError('Serial port not open');
    final completer = Completer<List<String>>();
    final lines = <String>[];
    final buffer = StringBuffer();
    late final StreamSubscription<Uint8List> sub;
    Timer? timer;
    sub = _port!.inputStream!.listen((Uint8List event) {
      buffer.write(String.fromCharCodes(event));
      var text = buffer.toString();
      while (text.contains(terminator)) {
        final idx = text.indexOf(terminator);
        lines.add(text.substring(0, idx).trim());
        text = text.substring(idx + terminator.length);
      }
      buffer.clear();
      buffer.write(text);
      // A block is complete once a line has been read and nothing partial is
      // left buffered.
      if (lines.isNotEmpty && text.isEmpty && !completer.isCompleted) {
        completer.complete(List.of(lines));
      }
    }, onError: (_) {
      if (!completer.isCompleted) completer.complete(List.of(lines));
    });
    timer = Timer(Duration(milliseconds: timeoutMs), () {
      if (!completer.isCompleted) {
        final tail = buffer.toString().trim();
        if (tail.isNotEmpty) lines.add(tail);
        completer.complete(List.of(lines));
      }
    });
    try {
      return await completer.future;
    } finally {
      timer.cancel();
      await sub.cancel();
    }
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
