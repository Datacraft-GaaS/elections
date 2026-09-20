import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:battery_plus/battery_plus.dart';
import 'package:flutter/material.dart';
import 'package:geolocator/geolocator.dart';
import 'package:http/http.dart' as http;

import 'package:path_provider/path_provider.dart';
import 'package:flutter/services.dart';

void main() {
  runApp(const TestHarnessApp());
}

class TestHarnessApp extends StatelessWidget {
  const TestHarnessApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Eel Test Harness',
      theme: ThemeData(
        primarySwatch: Colors.blue,
      ),
      home: const TestHarnessHome(),
    );
  }
}

class TestHarnessHome extends StatefulWidget {
  const TestHarnessHome({super.key});

  @override
  State<TestHarnessHome> createState() => _TestHarnessHomeState();
}

class _TestHarnessHomeState extends State<TestHarnessHome> {
  final List<String> _logs = [];
  final List<Map<String, dynamic>> _results = [];
  bool _isTesting = false;
  Map<String, dynamic> _deviceInfo = {};
  DateTime? _startTime;
  DateTime? _endTime;

  final MethodChannel _irisChannel = const MethodChannel('eel.bvi/iris');
  final MethodChannel _fingerprintChannel =
      const MethodChannel('eel/fingerprint');
  final MethodChannel _printerChannel = const MethodChannel('eel/bvi/printer');
  final MethodChannel _smsChannel = const MethodChannel('eel/modem/sms');
  final MethodChannel _scannerChannel =
      const MethodChannel('eel/bvi/document_scanner');
  final MethodChannel _cameraChannel = const MethodChannel('eel/bvi/camera');
  final MethodChannel _deviceChannel = const MethodChannel('eel/device/info');

  @override
  void initState() {
    super.initState();
    _log('Test harness ready');
    // Gather device info and start tests after a short delay
    Future.delayed(const Duration(seconds: 1), () async {
      _deviceInfo = await _getDeviceInfo();
      _startTime = DateTime.now();
      await _runAllTests();
      _endTime = DateTime.now();
      await _saveReport();
      if (mounted) {
        setState(() {
          _isTesting = false;
        });
      }
    });
  }

  Future<Map<String, dynamic>> _getDeviceInfo() async {
    try {
      final result =
          await _deviceChannel.invokeMethod<Map<String, dynamic>>('getInfo');
      if (result != null) {
        return result;
      }
    } on PlatformException catch (_) {
      // Fall back to Platform
    } on MissingPluginException {
      // No native implementation; use Platform
    }
    // Basic info from dart:io
    return {
      'os': Platform.operatingSystem,
      'osVersion': Platform.operatingSystemVersion,
      'hostname': Platform.localHostname,
      'model': 'unknown', // fallback
      'manufacturer': 'unknown',
    };
  }

  void _log(String message) {
    final timestamp = DateTime.now().toIso8601String();
    final line = '[$timestamp] $message';
    debugPrint(line);
    setState(() {
      _logs.add(line);
    });
  }


  /// Hardware-presence probe (eel_drivers `probeDetails`). Returns true when
  /// the check should keep FAIL semantics: hardware present but unusable, or
  /// a foreign plugin without the probe method. Returns false only when the
  /// plugin positively reports no such hardware attached (harness SKIP).
  /// Native maps arrive as Map<Object?, Object?>; the test bodies below
  /// historically cast to Map<String, dynamic>, which throws once a real
  /// plugin answers (previously MissingPluginException short-circuited).
  static Map<String, dynamic>? _strMap(Object? o) =>
      (o as Map?)?.map((k, v) => MapEntry(k.toString(), v));

  Future<bool> _present(MethodChannel channel) async {
    try {
      final probe = _strMap(await channel.invokeMethod('probeDetails'));
      return (probe ?? {})['present'] == true;
    } catch (_) {
      return true;
    }
  }

  Future<Map<String, dynamic>> _runTestWithTiming(
      Future<Map<String, dynamic>> Function() testFn,
      String featureName) async {
    final testStart = DateTime.now();
    _log('Starting $featureName test');
    Map<String, dynamic> result;
    try {
      result = await testFn();
    } catch (e) {
      result = {
        'feature': featureName,
        'status': 'ERROR',
        'details': e.toString(),
        'data': null
      };
    }
    final testEnd = DateTime.now();
    final durationMs = testEnd.difference(testStart).inMilliseconds;
    result['testStart'] = testStart.toIso8601String();
    result['testEnd'] = testEnd.toIso8601String();
    result['durationMs'] = durationMs;
    _log('$featureName test completed: ${result['status']} (${durationMs}ms)');
    return result;
  }

  Future<Map<String, dynamic>> _testIris() async {
    try {
      final result = await _irisChannel.invokeMethod<bool>('init');
      if (result == true) {
        // Attempt capture
        final captureResult = _strMap(await _irisChannel.invokeMethod('captureIris', {
          'userName': 'test',
          'eyeIndex': 0,
          'irexType': 7,
          'timeoutMs': 5000
        }));
        if (captureResult != null && captureResult['isValid'] == true) {
          return {
            'feature': 'Iris',
            'status': 'PASS',
            'details': 'Iris capture succeeded',
            'data': captureResult
          };
        } else {
          return {
            'feature': 'Iris',
            'status': 'FAIL',
            'details': 'Iris capture failed or invalid',
            'data': captureResult
          };
        }
            } else {
        if (!await _present(_irisChannel)) {
          return {
            'feature': 'Iris',
            'status': 'SKIP',
            'details': 'No iris hardware attached',
            'data': null
          };
        }
   return {
          'feature': 'Iris',
          'status': 'FAIL',
          'details': 'AntIris init failed',
          'data': null
        };
      }
    } on PlatformException catch (e) {
      return {
        'feature': 'Iris',
        'status': 'ERROR',
        'details': e.message ?? 'PlatformException',
        'data': null
      };
    } on MissingPluginException {
      return {
        'feature': 'Iris',
        'status': 'SKIP',
        'details': 'AntIris plugin not present on device',
        'data': null
      };
    } catch (e) {
      return {
        'feature': 'Iris',
        'status': 'ERROR',
        'details': e.toString(),
        'data': null
      };
    }
  }

  Future<Map<String, dynamic>> _testFingerprint() async {
    try {
      final initResult = await _fingerprintChannel.invokeMethod<bool>('init');
      if (initResult == true) {
        // Attempt capture
        final captureResult = _strMap(await _fingerprintChannel.invokeMethod('captureFingerprint',
                {'position': 'left_index', 'timeoutMs': 5000}));
        if (captureResult != null &&
            captureResult['isValid'] == true &&
            captureResult['quality'] != null &&
            (captureResult['quality'] as int) >= 40) {
          return {
            'feature': 'Fingerprint',
            'status': 'PASS',
            'details': 'Fingerprint capture succeeded',
            'data': captureResult
          };
        } else {
          return {
            'feature': 'Fingerprint',
            'status': 'FAIL',
            'details': 'Fingerprint capture failed or low quality',
            'data': captureResult
          };
        }
            } else {
        if (!await _present(_fingerprintChannel)) {
          return {
            'feature': 'Fingerprint',
            'status': 'SKIP',
            'details': 'No fingerprint hardware attached',
            'data': null
          };
        }
   return {
          'feature': 'Fingerprint',
          'status': 'FAIL',
          'details': 'Fingerprint init failed',
          'data': null
        };
      }
    } on PlatformException catch (e) {
      return {
        'feature': 'Fingerprint',
        'status': 'ERROR',
        'details': e.message ?? 'PlatformException',
        'data': null
      };
    } on MissingPluginException {
      return {
        'feature': 'Fingerprint',
        'status': 'SKIP',
        'details': 'Fingerprint plugin not present on device',
        'data': null
      };
    } catch (e) {
      return {
        'feature': 'Fingerprint',
        'status': 'ERROR',
        'details': e.toString(),
        'data': null
      };
    }
  }

  Future<Map<String, dynamic>> _testPrinter() async {
    try {
      final result = await _printerChannel.invokeMethod<bool>('isAvailable');
      if (result == true) {
        // Try a simple print job
        final printResult =
            await _printerChannel.invokeMethod<bool>('printTestPage');
        if (printResult == true) {
          return {
            'feature': 'Printer',
            'status': 'PASS',
            'details': 'Printer test page printed successfully',
            'data': null
          };
        } else {
          return {
            'feature': 'Printer',
            'status': 'FAIL',
            'details': 'Printer test page failed',
            'data': null
          };
        }
            } else {
        if (!await _present(_printerChannel)) {
          return {
            'feature': 'Printer',
            'status': 'SKIP',
            'details': 'No printer hardware attached',
            'data': null
          };
        }
   return {
          'feature': 'Printer',
          'status': 'FAIL',
          'details': 'Printer not available',
          'data': null
        };
      }
    } on PlatformException catch (e) {
      return {
        'feature': 'Printer',
        'status': 'ERROR',
        'details': e.message ?? 'PlatformException',
        'data': null
      };
    } on MissingPluginException {
      return {
        'feature': 'Printer',
        'status': 'SKIP',
        'details': 'Printer plugin not present on device',
        'data': null
      };
    } catch (e) {
      return {
        'feature': 'Printer',
        'status': 'ERROR',
        'details': e.toString(),
        'data': null
      };
    }
  }

  Future<Map<String, dynamic>> _testSMS() async {
    try {
      final result = await _smsChannel.invokeMethod<bool>('isAvailable');
      if (result == true) {
        // Try to send a test SMS (we'll send to a dummy number or just test AT)
        final smsResult = await _smsChannel.invokeMethod<bool>('sendTestSms');
        if (smsResult == true) {
          return {
            'feature': 'SMS Modem',
            'status': 'PASS',
            'details': 'Test SMS sent successfully',
            'data': null
          };
        } else {
          return {
            'feature': 'SMS Modem',
            'status': 'FAIL',
            'details': 'Test SMS failed to send',
            'data': null
          };
        }
            } else {
        if (!await _present(_smsChannel)) {
          return {
            'feature': 'SMS Modem',
            'status': 'SKIP',
            'details': 'No SMS modem hardware attached',
            'data': null
          };
        }
   return {
          'feature': 'SMS Modem',
          'status': 'FAIL',
          'details': 'SMS modem not available',
          'data': null
        };
      }
    } on PlatformException catch (e) {
      return {
        'feature': 'SMS Modem',
        'status': 'ERROR',
        'details': e.message ?? 'PlatformException',
        'data': null
      };
    } on MissingPluginException {
      return {
        'feature': 'SMS Modem',
        'status': 'SKIP',
        'details': 'SMS modem plugin not present on device',
        'data': null
      };
    } catch (e) {
      return {
        'feature': 'SMS Modem',
        'status': 'ERROR',
        'details': e.toString(),
        'data': null
      };
    }
  }

  Future<Map<String, dynamic>> _testScanner() async {
    try {
      final result = await _scannerChannel.invokeMethod<bool>('isAvailable');
      if (result == true) {
        // Attempt a scan
        final scanResult = _strMap(await _scannerChannel.invokeMethod('scanDocument',
                {'source': 'flatbed', 'resolution': 300, 'format': 'pdf'}));
        if (scanResult != null && scanResult['success'] == true) {
          return {
            'feature': 'Document Scanner',
            'status': 'PASS',
            'details': 'Document scan succeeded',
            'data': scanResult
          };
        } else {
          return {
            'feature': 'Document Scanner',
            'status': 'FAIL',
            'details': 'Document scan failed',
            'data': scanResult
          };
        }
            } else {
        if (!await _present(_scannerChannel)) {
          return {
            'feature': 'Document Scanner',
            'status': 'SKIP',
            'details': 'No scanner hardware attached',
            'data': null
          };
        }
   return {
          'feature': 'Document Scanner',
          'status': 'FAIL',
          'details': 'Document scanner not available',
          'data': null
        };
      }
    } on PlatformException catch (e) {
      return {
        'feature': 'Document Scanner',
        'status': 'ERROR',
        'details': e.message ?? 'PlatformException',
        'data': null
      };
    } on MissingPluginException {
      return {
        'feature': 'Document Scanner',
        'status': 'SKIP',
        'details': 'Document scanner plugin not present on device',
        'data': null
      };
    } catch (e) {
      return {
        'feature': 'Document Scanner',
        'status': 'ERROR',
        'details': e.toString(),
        'data': null
      };
    }
  }

  Future<Map<String, dynamic>> _testCamera() async {
    try {
      final result = await _cameraChannel.invokeMethod<bool>('isAvailable');
      if (result == true) {
        // Try to capture a preview frame
        final captureResult = _strMap(await _cameraChannel.invokeMethod('captureImage',
                {'width': 640, 'height': 480, 'format': 'jpeg'}));
        if (captureResult != null &&
            captureResult['success'] == true &&
            captureResult['imageBytes'] != null) {
          return {
            'feature': 'Camera',
            'status': 'PASS',
            'details': 'Camera image captured successfully',
            'data': {
              'imageBytes': captureResult['imageBytes'],
              'width': captureResult['width'],
              'height': captureResult['height']
            }
          };
        } else {
          return {
            'feature': 'Camera',
            'status': 'FAIL',
            'details': 'Camera capture failed',
            'data': captureResult
          };
        }
            } else {
        if (!await _present(_cameraChannel)) {
          return {
            'feature': 'Camera',
            'status': 'SKIP',
            'details': 'No camera hardware attached',
            'data': null
          };
        }
   return {
          'feature': 'Camera',
          'status': 'FAIL',
          'details': 'Camera not available',
          'data': null
        };
      }
    } on PlatformException catch (e) {
      return {
        'feature': 'Camera',
        'status': 'ERROR',
        'details': e.message ?? 'PlatformException',
        'data': null
      };
    } on MissingPluginException {
      return {
        'feature': 'Camera',
        'status': 'SKIP',
        'details': 'Camera plugin not present on device',
        'data': null
      };
    } catch (e) {
      return {
        'feature': 'Camera',
        'status': 'ERROR',
        'details': e.toString(),
        'data': null
      };
    }
  }

  Future<Map<String, dynamic>> _testGNSS() async {
    try {
      // Request location permission
      bool serviceEnabled = await Geolocator.isLocationServiceEnabled();
      if (!serviceEnabled) {
        return {
          'feature': 'GNSS',
          'status': 'FAIL',
          'details': 'Location services disabled',
          'data': null
        };
      }
      LocationPermission permission = await Geolocator.checkPermission();
      if (permission == LocationPermission.denied) {
        permission = await Geolocator.requestPermission();
        if (permission == LocationPermission.denied) {
          return {
            'feature': 'GNSS',
            'status': 'FAIL',
            'details': 'Location permissions denied',
            'data': null
          };
        }
      }
      if (permission == LocationPermission.deniedForever) {
        return {
          'feature': 'GNSS',
          'status': 'FAIL',
          'details': 'Location permissions permanently denied',
          'data': null
        };
      }
      LocationSettings locationSettings = const LocationSettings(
        accuracy: LocationAccuracy.low,
      );
      // Bounded wait: emulators and indoor factory benches may never produce
      // a fix; without a timeout the whole suite hangs and no report is
      // ever written.
      Position position = await Geolocator.getCurrentPosition(
              locationSettings: locationSettings)
          .timeout(const Duration(seconds: 30));
      return {
        'feature': 'GNSS',
        'status': 'PASS',
        'details': 'GNSS fix obtained',
        'data': {
          'latitude': position.latitude,
          'longitude': position.longitude,
          'accuracy': position.accuracy,
          'timestamp': position.timestamp.toIso8601String()
        }
      };
    } on TimeoutException {
      return {
        'feature': 'GNSS',
        'status': 'FAIL',
        'details': 'GNSS fix timeout (no fix within 30s)',
        'data': null
      };
    } catch (e) {
      return {
        'feature': 'GNSS',
        'status': 'ERROR',
        'details': e.toString(),
        'data': null
      };
    }
  }

  Future<Map<String, dynamic>> _testNTS() async {
    try {
      // Attempt to reach a known NTS endpoint; we'll use a placeholder
      // In real deployment, replace with actual NTS-KE authority URL.
      final uri = Uri.https('time.eel.local', '/v1/time/nts');
      final response = await http.get(uri).timeout(const Duration(seconds: 5));
      if (response.statusCode == 200) {
        final data = jsonDecode(response.body);
        return {
          'feature': 'NTS',
          'status': 'PASS',
          'details': 'NTS time retrieved',
          'data': data
        };
      } else {
        return {
          'feature': 'NTS',
          'status': 'FAIL',
          'details': 'NTS endpoint returned ${response.statusCode}',
          'data': null
        };
      }
    } catch (e) {
      return {
        'feature': 'NTS',
        'status': 'ERROR',
        'details': e.toString(),
        'data': null
      };
    }
  }

  Future<Map<String, dynamic>> _testBattery() async {
    try {
      final battery = Battery();
      final state = await battery.batteryState;
      final level = await battery.batteryLevel;
      return {
        'feature': 'Battery',
        'status': 'PASS',
        'details': 'Battery info retrieved',
        'data': {'state': state.toString(), 'level': level}
      };
    } catch (e) {
      return {
        'feature': 'Battery',
        'status': 'ERROR',
        'details': e.toString(),
        'data': null
      };
    }
  }

  Future<void> _runAllTests() async {
    setState(() {
      _isTesting = true;
      _results.clear();
    });
    _log('Starting full test suite');

    final List<MapEntry<String, Future<Map<String, dynamic>> Function()>>
        tests = [
      MapEntry('Iris', _testIris),
      MapEntry('Fingerprint', _testFingerprint),
      MapEntry('Printer', _testPrinter),
      MapEntry('SMS Modem', _testSMS),
      MapEntry('Document Scanner', _testScanner),
      MapEntry('Camera', _testCamera),
      MapEntry('GNSS', _testGNSS),
      MapEntry('NTS', _testNTS),
      MapEntry('Battery', _testBattery)
    ];

    for (final entry in tests) {
      final result = await _runTestWithTiming(entry.value, entry.key);
      setState(() {
        _results.add(result);
      });
    }

    _log('All tests completed');
  }

  Future<void> _saveReport() async {
    try {
      final dir = await getApplicationDocumentsDirectory();
      final file = File('${dir.path}/test_report.json');
      final report = {
        'timestamp': DateTime.now().toIso8601String(),
        'startTime': _startTime?.toIso8601String(),
        'endTime': _endTime?.toIso8601String(),
        'overallDurationMs': _endTime != null && _startTime != null
            ? _endTime!.difference(_startTime!).inMilliseconds
            : null,
        'deviceInfo': _deviceInfo,
        'results': _results
      };
      await file.writeAsString(jsonEncode(report));
      _log('Report saved to ${file.path}');
    } catch (e) {
      _log('Failed to save report: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Eel Test Harness'),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Center(
          child: _isTesting
              ? const Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    CircularProgressIndicator(),
                    SizedBox(height: 16),
                    Text('Running tests...', style: TextStyle(fontSize: 18)),
                  ],
                )
              : _results.isEmpty
                  ? const Text('No test results yet.',
                      style: TextStyle(fontSize: 16))
                  : ListView.builder(
                      itemCount: _results.length,
                      itemBuilder: (context, index) {
                        final r = _results[index];
                        return ListTile(
                          title: Text('${r['feature']} - ${r['status']}'),
                          subtitle: Text(
                              '${r['details']} (${r['durationMs'] ?? 0} ms)'),
                        );
                      },
                    ),
        ),
      ),
    );
  }
}
