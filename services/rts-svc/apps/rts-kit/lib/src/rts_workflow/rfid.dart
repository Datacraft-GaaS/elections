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

/// Ballot-lot RFID scan integrating with M39 ballot-serial-chain.
///
/// The physical RFID reader is accessed via a device SDK that varies by
/// hardware platform (KIEMS tablet vs. legacy Android). This module defines
/// the abstract [RfidReader] interface and a stub implementation.
///
/// Production wiring:
///   - New KIEMS kits: wire [RfidReader] to the OEM NFC/RFID SDK via a
///     platform channel (MethodChannel 'eel.rts/rfid').
///   - Legacy Android: same platform channel, different native implementation.
///   - The scanned [BallotLotTag] is forwarded to the M39 ballot-serial-chain
///     via a `serial_scanned.v1` event (defined in eel-events-rs schema).
///
/// The stub simulates a scan with a random lot ID for testing.
library;

import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:logging/logging.dart';

import '../a11y/workflow_semantics.dart';

final _log = Logger('eel.rts.rfid');

// ---- Domain -----------------------------------------------------------------

class BallotLotTag {
  const BallotLotTag({
    required this.lotId,
    required this.stationCode,
    required this.issuedAt,
    required this.scannedAt,
  });

  final String lotId;
  final String stationCode;
  final DateTime issuedAt;
  final DateTime scannedAt;

  Map<String, Object?> toJson() => {
        'lot_id': lotId,
        'station_code': stationCode,
        'issued_at': issuedAt.toIso8601String(),
        'scanned_at': scannedAt.toIso8601String(),
      };
}

// ---- Abstract reader --------------------------------------------------------

abstract class RfidReader {
  /// Scan one RFID tag and return the parsed [BallotLotTag].
  ///
  /// Throws [RfidScanException] if the tag is invalid, unrecognised, or
  /// does not match the expected station code.
  Future<BallotLotTag> scan({required String expectedStationCode});

  /// Check whether RFID hardware is available on this device.
  Future<bool> isAvailable();
}

class RfidScanException implements Exception {
  const RfidScanException(this.message);
  final String message;
  @override
  String toString() => 'RfidScanException: $message';
}

// ---- Platform channel reader (production) -----------------------------------

/// Routes RFID scan calls to native via MethodChannel.
class PlatformChannelRfidReader implements RfidReader {
  static const _channel = MethodChannel('eel.rts/rfid');

  @override
  Future<bool> isAvailable() async {
    try {
      return await _channel.invokeMethod<bool>('isAvailable') ?? false;
    } on MissingPluginException {
      return false;
    }
  }

  @override
  Future<BallotLotTag> scan({required String expectedStationCode}) async {
    final result = await _channel.invokeMapMethod<String, Object?>(
      'scan',
      {'expectedStationCode': expectedStationCode},
    );
    if (result == null) throw const RfidScanException('No tag returned');
    return BallotLotTag(
      lotId: result['lot_id'] as String,
      stationCode: result['station_code'] as String,
      issuedAt: DateTime.parse(result['issued_at'] as String),
      scannedAt: DateTime.now().toUtc(),
    );
  }
}

// ---- Stub reader (dev / test) -----------------------------------------------

class RfidReaderStub implements RfidReader {
  static final _rng = Random.secure();

  @override
  Future<bool> isAvailable() async => true;

  @override
  Future<BallotLotTag> scan({required String expectedStationCode}) async {
    // Simulate 800 ms NFC/RFID read round-trip.
    await Future<void>.delayed(const Duration(milliseconds: 800));
    final lotId = 'LOT-${_rng.nextInt(999999).toString().padLeft(6, '0')}';
    _log.info('RFID stub scan: lotId=$lotId station=$expectedStationCode');
    return BallotLotTag(
      lotId: lotId,
      stationCode: expectedStationCode,
      issuedAt: DateTime.now().toUtc().subtract(const Duration(hours: 1)),
      scannedAt: DateTime.now().toUtc(),
    );
  }
}

// ---- Provider ---------------------------------------------------------------

final rfidReaderProvider = Provider<RfidReader>((ref) {
  // Production: uses PlatformChannelRfidReader (native SDK).
  // Dev/test override via DI; stub removed.
  return PlatformChannelRfidReader();
});

// ---- Screen -----------------------------------------------------------------

class RfidScanScreen extends ConsumerStatefulWidget {
  const RfidScanScreen({super.key, required this.stationCode});
  final String stationCode;

  @override
  ConsumerState<RfidScanScreen> createState() => _RfidScanScreenState();
}

class _RfidScanScreenState extends ConsumerState<RfidScanScreen> {
  BallotLotTag? _tag;
  bool _scanning = false;
  String? _error;

  Future<void> _scan() async {
    setState(() {
      _scanning = true;
      _error = null;
    });
    try {
      final reader = ref.read(rfidReaderProvider);
      final tag = await reader.scan(expectedStationCode: widget.stationCode);
      if (mounted) setState(() => _tag = tag);
    } on RfidScanException catch (e) {
      if (mounted) setState(() => _error = e.message);
    } catch (e) {
      if (mounted) setState(() => _error = e.toString());
    } finally {
      if (mounted) setState(() => _scanning = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final tag = _tag;
    return Scaffold(
      appBar: AppBar(
        title: const EelSemanticsLabel(
          label: 'Scan ballot lot RFID tag',
          child: Text('Ballot Lot RFID'),
        ),
      ),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Semantics(
              label: 'Station: ${widget.stationCode}',
              child: Text('Station: ${widget.stationCode}'),
            ),
            const SizedBox(height: 24),
            if (_error != null)
              WfSemanticsBanner(
                message: _error!,
                role: 'error',
                child: Text(_error!, style: const TextStyle(color: Colors.red)),
              ),
            if (tag != null)
              Semantics(
                liveRegion: true,
                label: 'Lot scanned: ${tag.lotId}',
                child: Card(
                  child: Padding(
                    padding: const EdgeInsets.all(12),
                    child: Column(
                      children: [
                        const Text('Lot scanned'),
                        Text(tag.lotId, style: const TextStyle(fontWeight: FontWeight.bold)),
                      ],
                    ),
                  ),
                ),
              ),
            const SizedBox(height: 24),
            EelMinHitTarget(
              semanticsLabel: tag == null ? 'Scan ballot lot RFID' : 'Re-scan RFID',
              onTap: _scanning ? null : _scan,
              child: FilledButton.icon(
                onPressed: _scanning ? null : _scan,
                icon: _scanning
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Icon(Icons.nfc),
                label: Text(_scanning ? 'Scanning...' : (tag == null ? 'Scan RFID' : 'Re-scan')),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
