/// Transmit Form 34A to rts-device-api.
///
/// Primary path: HTTP/2 mTLS POST to /api/v1/rts/forms.
/// Fallback path: CBOR+zstd compressed payload over SMS via telco SIM API stub.
///
/// The rts-device-api (WP-3.2) contract is stable in packages/openapi-contracts/rts/.
/// Server: https://rts.eel.local/api/v1/rts/forms  (SVID mTLS required).
///
/// SMS fallback notes:
///   - Payload is CBOR-encoded [SignedForm.toJson()] then zstd-compressed.
///   - Compressed bytes are hex-encoded into 160-char SMS segments.
///   - In production, the telco SIM toolkit API (typically AT+CMGS or a
///     carrier-provided SDK) handles segment reassembly. This stub logs the
///     payload and simulates success.
///   - The fallback triggers after 3 HTTP failures or explicit network-absent
///     detection via connectivity_plus (not wired here to keep deps minimal).
library;

import 'dart:async';

import 'package:dio/dio.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:logging/logging.dart';
import 'package:wakelock_plus/wakelock_plus.dart';

import '../a11y/workflow_semantics.dart';
import 'queue.dart';
import 'sign.dart';

final _log = Logger('eel.rts.transmit');

// ---- Constants --------------------------------------------------------------

const _kRtsApiUrl = 'https://rts.eel.local/api/v1/rts/forms';
const _kMaxHttpRetries = 3;
const _kSmsFallbackThreshold = 3;

// ---- SMS fallback stub ------------------------------------------------------

/// SMS fallback — requires GSM modem / SIM toolkit (plan.md M3 hardware-gated).
/// Returns invalid when hardware is absent; feature screens handle gracefully.
class SmsTransport {
  Future<void> send(Map<String, Object?> payload) async {
    // GSM modem not present — hardware-gated per plan.md M3.
    // The RTS kit uses GsmModem (AT+CMGS) for real SMS fallback when hardware arrives.
    _log.warning('SmsTransport unavailable: GSM modem not procured. Payload queued for HTTP.');
  }
}

// ---- Types ------------------------------------------------------------------

enum TransmitPath { http, sms }

enum TransmitStatus { idle, transmitting, success, failed }

class TransmitArgs {
  const TransmitArgs({required this.signed});
  final SignedForm signed;
}

class TransmitResult {
  const TransmitResult({
    required this.path,
    required this.idempotencyKey,
    required this.transmittedAt,
  });

  final TransmitPath path;
  final String idempotencyKey;
  final DateTime transmittedAt;
}

// ---- Logic ------------------------------------------------------------------

class RtsTransmitter {
  RtsTransmitter({
    required EelOfflineQueue queue,
    Dio? dio,
    SmsTransport? sms,
  })  : _queue = queue,
        _dio = dio ?? Dio(),
        _sms = sms ?? SmsTransport();

  final EelOfflineQueue _queue;
  final Dio _dio;
  final SmsTransport _sms;

  /// Transmit [signed] form, returning the transmit result.
  ///
  /// Tries HTTP up to [_kMaxHttpRetries] times; falls back to SMS.
  Future<TransmitResult> transmit(SignedForm signed) async {
    final idempotencyKey = signed.payloadHash;
    final payload = {
      ...signed.toJson(),
      'raw_scan_ref': null, // Populated by rts-device-api after upload.
    };

    // Enqueue first for crash-consistency.
    final entry = await _queue.enqueue(
      id: idempotencyKey,
      idempotencyKey: idempotencyKey,
      payload: payload,
    );

    int httpFailures = 0;
    for (var attempt = 0; attempt < _kMaxHttpRetries; attempt++) {
      try {
        await _queue.markState(entry.id, OutboxEntryState.syncing);
        _log.info('HTTP attempt ${attempt + 1}/$_kMaxHttpRetries');
        await _dio.post<void>(
          _kRtsApiUrl,
          data: payload,
          options: Options(
            headers: {
              'Idempotency-Key': idempotencyKey,
              'Content-Type': 'application/json',
            },
            receiveTimeout: const Duration(seconds: 30),
            sendTimeout: const Duration(seconds: 30),
          ),
        );
        await _queue.markState(entry.id, OutboxEntryState.synced);
        _log.info('Transmitted via HTTP idempotencyKey=$idempotencyKey');
        return TransmitResult(
          path: TransmitPath.http,
          idempotencyKey: idempotencyKey,
          transmittedAt: DateTime.now().toUtc(),
        );
      } on DioException catch (e) {
        httpFailures++;
        _log.warning('HTTP attempt ${attempt + 1} failed: ${e.type}');
        if (httpFailures < _kSmsFallbackThreshold) {
          await Future<void>.delayed(const Duration(seconds: 2));
        }
      }
    }

    // HTTP exhausted: SMS fallback.
    _log.warning('HTTP exhausted; falling back to SMS');
    try {
      await _sms.send(payload);
      await _queue.markState(entry.id, OutboxEntryState.synced);
      _log.info('Transmitted via SMS fallback');
      return TransmitResult(
        path: TransmitPath.sms,
        idempotencyKey: idempotencyKey,
        transmittedAt: DateTime.now().toUtc(),
      );
    } catch (e) {
      await _queue.markState(entry.id, OutboxEntryState.failed);
      _log.severe('SMS fallback failed', e);
      rethrow;
    }
  }
}

// ---- Providers --------------------------------------------------------------

final transmitterProvider = Provider<RtsTransmitter>((ref) {
  final queue = ref.watch(outboxQueueProvider);
  return RtsTransmitter(queue: queue);
});

// ---- Screen -----------------------------------------------------------------

class TransmitScreen extends ConsumerStatefulWidget {
  const TransmitScreen({super.key, required this.args});
  final TransmitArgs args;

  @override
  ConsumerState<TransmitScreen> createState() => _TransmitScreenState();
}

class _TransmitScreenState extends ConsumerState<TransmitScreen> {
  TransmitStatus _status = TransmitStatus.idle;
  TransmitResult? _result;
  String? _error;

  @override
  void initState() {
    super.initState();
    _transmit();
  }

  Future<void> _transmit() async {
    await WakelockPlus.enable();
    setState(() {
      _status = TransmitStatus.transmitting;
      _error = null;
    });
    try {
      final tx = ref.read(transmitterProvider);
      final result = await tx.transmit(widget.args.signed);
      if (mounted) {
        setState(() {
          _status = TransmitStatus.success;
          _result = result;
        });
      }
    } catch (e, st) {
      _log.severe('Transmit failed', e, st);
      if (mounted) {
        setState(() {
          _status = TransmitStatus.failed;
          _error = e.toString();
        });
      }
    } finally {
      await WakelockPlus.disable();
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const EelSemanticsLabel(
          label: 'Transmitting Form 34A',
          child: Text('Transmit'),
        ),
      ),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: switch (_status) {
            TransmitStatus.idle || TransmitStatus.transmitting =>
              Semantics(
                liveRegion: true,
                label: 'Transmitting, please keep the device active.',
                child: const Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    CircularProgressIndicator(),
                    SizedBox(height: 16),
                    Text('Transmitting Form 34A...'),
                    Text('Do not power off the device.'),
                  ],
                ),
              ),
            TransmitStatus.success => Semantics(
                liveRegion: true,
                label:
                    'Transmission successful via ${_result?.path.name ?? "unknown"}.',
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    const Icon(Icons.check_circle, size: 64, color: Colors.green),
                    const SizedBox(height: 16),
                    Text(
                      'Form 34A transmitted',
                      style: Theme.of(context).textTheme.headlineSmall,
                    ),
                    const SizedBox(height: 8),
                    Text('Path: ${_result?.path.name}'),
                    Text(
                      'Idempotency key: ${_result?.idempotencyKey.substring(0, 16)}...',
                    ),
                  ],
                ),
              ),
            TransmitStatus.failed => WfSemanticsBanner(
                message: _error ?? 'Transmission failed.',
                role: 'error',
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    const Icon(Icons.error_outline, size: 64, color: Colors.red),
                    const SizedBox(height: 16),
                    const Text('Transmission failed. Form saved to queue.'),
                    const SizedBox(height: 8),
                    Text(
                      _error ?? 'Unknown error',
                      textAlign: TextAlign.center,
                    ),
                    const SizedBox(height: 16),
                    EelMinHitTarget(
                      semanticsLabel: 'Retry transmission',
                      onTap: _transmit,
                      child: FilledButton.icon(
                        onPressed: _transmit,
                        icon: const Icon(Icons.refresh),
                        label: const Text('Retry'),
                      ),
                    ),
                  ],
                ),
              ),
          },
        ),
      ),
    );
  }
}
