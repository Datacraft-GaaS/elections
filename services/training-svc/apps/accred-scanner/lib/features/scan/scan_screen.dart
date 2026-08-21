import 'package:dio/dio.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import '../crl/crl_repository.dart';
import 'scan_result.dart';

final _crlRepoProvider = Provider((ref) {
  return CrlRepository(
    dio: Dio(BaseOptions(baseUrl: 'https://accred.eel.iebc.go.ke')),
  );
});

final _dioProvider = Provider((ref) {
  return Dio(BaseOptions(
    baseUrl: 'https://accred.eel.iebc.go.ke',
    connectTimeout: const Duration(milliseconds: 300),
    receiveTimeout: const Duration(milliseconds: 300),
  ));
});

class ScanScreen extends HookConsumerWidget {
  const ScanScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final controller = MobileScannerController();
    bool processing = false;

    return Scaffold(
      appBar: AppBar(title: const Text('Scan Badge')),
      body: MobileScanner(
        controller: controller,
        onDetect: (capture) async {
          if (processing) return;
          final barcode = capture.barcodes.firstOrNull;
          if (barcode == null || barcode.rawValue == null) return;
          processing = true;

          final qrValue = barcode.rawValue!;
          final serial = _extractSerial(qrValue);
          if (serial == null) {
            processing = false;
            return;
          }

          final result = await _checkBadge(
            serial: serial,
            ref: ref,
          );

          if (context.mounted) {
            controller.stop();
            context.go('/scan/result', extra: result);
          }
        },
      ),
    );
  }

  String? _extractSerial(String qrValue) {
    // QR encodes a JWT; extract 'sub' claim from payload
    // Format: header.payload.signature (base64url)
    try {
      final parts = qrValue.split('.');
      if (parts.length != 3) return qrValue; // treat as raw serial
      // Decode payload
      final payload = parts[1];
      final padded = payload.padRight(
        payload.length + (4 - payload.length % 4) % 4,
        '=',
      );
      final decoded = String.fromCharCodes(
        Uri.decodeComponent(
          padded.replaceAll('-', '+').replaceAll('_', '/'),
        ).codeUnits,
      );
      // Simple extraction: find "sub":"<value>"
      final match = RegExp(r'"sub"\s*:\s*"([^"]+)"').firstMatch(decoded);
      return match?.group(1) ?? qrValue;
    } catch (_) {
      return qrValue;
    }
  }

  Future<ScanResult> _checkBadge({
    required String serial,
    required WidgetRef ref,
  }) async {
    final crlRepo = ref.read(_crlRepoProvider);
    final dio = ref.read(_dioProvider);

    final isRevoked = await crlRepo.isRevoked(serial);

    ScanStatus liveStatus = isRevoked ? ScanStatus.revoked : ScanStatus.active;
    List<String> zones = [];
    bool fromCache = true;

    try {
      final resp = await dio.get<Map<String, dynamic>>(
        '/v1/badges/$serial/verify',
      );
      final data = resp.data!;
      liveStatus = _parseStatus(data['status'] as String? ?? 'active');
      zones = (data['zones'] as List<dynamic>? ?? [])
          .map((e) => e as String)
          .toList();
      fromCache = false;
    } on DioException {
      // Timeout or offline: use CRL result
    }

    return ScanResult(
      serial: serial,
      status: liveStatus,
      zones: zones,
      checkedAt: DateTime.now(),
      isFromCacheOnly: fromCache,
    );
  }

  ScanStatus _parseStatus(String s) {
    switch (s.toLowerCase()) {
      case 'revoked':
        return ScanStatus.revoked;
      case 'suspended':
        return ScanStatus.suspended;
      default:
        return ScanStatus.active;
    }
  }
}
