import 'package:eel_drivers/eel_drivers.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('channel inventory matches harness + kit-sdk contracts', () {
    expect(
      EelDriverChannels.all,
      containsAll([
        'eel.bvi/iris',
        'eel/iris',
        'eel/fingerprint',
        'eel/bvi/printer',
        'eel/modem/sms',
        'eel/bvi/document_scanner',
        'eel/bvi/camera',
      ]),
    );
  });
}
