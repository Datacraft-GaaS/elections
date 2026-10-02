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

/// Accred-Scanner Scan Screen — Beautiful, ergonomic, accessible.
library;
import 'package:dio/dio.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import '../crl/crl_repository.dart';
import 'scan_result.dart';

final _crlRepoProvider = Provider((ref) {
  return CrlRepository(
    dio: Dio(BaseOptions(baseUrl: 'https://accred.eel.example')),
  );
});

final _dioProvider = Provider((ref) {
  return Dio(BaseOptions(
    baseUrl: 'https://accred.eel.example',
    connectTimeout: const Duration(milliseconds: 300),
    receiveTimeout: const Duration(milliseconds: 300),
  ));
});

class ScanScreen extends HookConsumerWidget {
  const ScanScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final controller = MobileScannerController();

    return EelScaffold(
      title: 'Scan Badge',
      variant: EelScaffoldVariant.scrollable,
      padding: EdgeInsets.zero,
      body: MobileScanner(
        controller: controller,
        onDetect: (capture) async {
          final barcode = capture.barcodes.firstOrNull;
          if (barcode == null || barcode.rawValue == null) return;

          final qrValue = barcode.rawValue!;
          final serial = _extractSerial(qrValue);
          if (serial == null) return;

          final result = await _checkBadge(serial: serial, ref: ref);

          if (context.mounted) {
            controller.stop();
            context.go('/scan/result', extra: result);
          }
        },
      ),
    );
  }

  String? _extractSerial(String qrValue) {
    try {
      final parts = qrValue.split('.');
      if (parts.length != 3) return qrValue;
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
      final resp = await dio.get<Map<String, dynamic>>('/v1/badges/$serial/verify');
      final data = resp.data!;
      liveStatus = _parseStatus(data['status'] as String? ?? 'active');
      zones = (data['zones'] as List<dynamic>? ?? []).map((e) => e as String).toList();
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
    return switch (s.toLowerCase()) {
      'revoked' => ScanStatus.revoked,
      'suspended' => ScanStatus.suspended,
      _ => ScanStatus.active,
    };
  }
}
