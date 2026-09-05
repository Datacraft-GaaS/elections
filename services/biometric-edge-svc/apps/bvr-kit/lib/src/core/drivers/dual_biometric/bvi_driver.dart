/// BVI standalone driver (re-exported for dual biometric flow).
import 'package:flutter/services.dart';
import 'antiris/antiris_driver.dart';

class AntIrisDriver {
  static const _channel = MethodChannel('eel.bvi/iris');
  Future<AntIrisResult> captureIris({String userName = 'user', int eyeIndex = 0, int irexType = 7, int timeoutMs = 30000}) async {
    try {
      final r = await _channel.invokeMethod('captureIris', {
        'userName': userName, 'eyeIndex': eyeIndex, 'irexType': irexType, 'timeoutMs': timeoutMs,
      });
      return AntIrisResult(isValid: r['isValid'] == true);
    } on MissingPluginException {
      return AntIrisResult(isValid: false, error: 'antiris_plugin_not_registered');
    }
  }
  Future<bool> match(String t1, String t2) async => true; // stub
}
