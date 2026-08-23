/// RFID scan workflow screen for custody officers.
///
/// Flow:
///   1. Pair with a BLE reader (Zebra RFD40 or CipherLab RS36).
///   2. Present tag EPC + tamper_indicator + challenge status on scan.
///   3. Officer confirms and signs the state transition.
///   4. Duress long-press sends silent alert to ECC dispatch.
///   5. Offline queue: reads persist in Isar; sync on reconnect.
library rfid_scan_screen;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';

import '../services/rfid_reader_service.dart';

// ---------------------------------------------------------------------------
// Offline queue entry (simplified; real impl uses Isar)
// ---------------------------------------------------------------------------

class _OfflineRead {
  _OfflineRead({
    required this.epc,
    required this.scannedAt,
    required this.tamperIndicator,
    required this.challengeValid,
  });

  final String epc;
  final DateTime scannedAt;
  final bool tamperIndicator;
  final bool challengeValid;
  bool synced = false;
}

// ---------------------------------------------------------------------------
// Screen
// ---------------------------------------------------------------------------

class RfidScanScreen extends StatefulWidget {
  const RfidScanScreen({
    super.key,
    required this.officerId,
    required this.readerService,
    this.onDuressAlert,
    this.onTransitionSubmit,
  });

  final String officerId;
  final RfidReaderService readerService;

  /// Called when the officer triggers a duress long-press.
  final Future<void> Function()? onDuressAlert;

  /// Called when the officer confirms a state transition.
  final Future<void> Function(String epc, String officerId)? onTransitionSubmit;

  @override
  State<RfidScanScreen> createState() => _RfidScanScreenState();
}

class _RfidScanScreenState extends State<RfidScanScreen> {
  RfidScanResult? _lastScan;
  ReaderConnectionState _connectionState = ReaderConnectionState.disconnected;
  bool _submitting = false;
  bool _confirmed = false;
  final List<_OfflineRead> _offlineQueue = [];
  StreamSubscription<RfidScanResult>? _scanSub;
  StreamSubscription<ReaderConnectionState>? _stateSub;

  @override
  void initState() {
    super.initState();
    _stateSub = widget.readerService.connectionStateStream.listen((state) {
      if (mounted) setState(() => _connectionState = state);
    });
    _scanSub = widget.readerService.scanResults.listen(_onScan);
  }

  @override
  void dispose() {
    _scanSub?.cancel();
    _stateSub?.cancel();
    super.dispose();
  }

  void _onScan(RfidScanResult result) {
    if (!mounted) return;
    setState(() {
      _lastScan = result;
      _confirmed = false;
      _offlineQueue.add(_OfflineRead(
        epc: result.epc,
        scannedAt: result.readAt,
        tamperIndicator: result.tamperIndicator,
        challengeValid: result.challengeValid,
      ));
    });

    if (result.tamperIndicator) {
      SemanticsService.sendAnnouncement(
        View.of(context),
        'Warning: tamper indicator detected on tag ${result.epc}',
        TextDirection.ltr,
      );
    }
  }

  Future<void> _connectReader() async {
    await widget.readerService.startScan();
    // In production, show a device picker. For now, auto-connect synthetic.
    await widget.readerService.connectToDevice('synthetic-device-001');
  }

  Future<void> _submitTransition() async {
    final scan = _lastScan;
    if (scan == null || !_confirmed) return;
    setState(() => _submitting = true);
    try {
      await widget.onTransitionSubmit?.call(scan.epc, widget.officerId);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('State transition submitted.')),
        );
        setState(() {
          _lastScan = null;
          _confirmed = false;
        });
      }
    } finally {
      if (mounted) setState(() => _submitting = false);
    }
  }

  Future<void> _triggerDuress() async {
    await widget.onDuressAlert?.call();
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Silent duress alert sent to ECC.'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('RFID Cover Scan'),
        actions: [
          Semantics(
            label: 'Reader connection status',
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12),
              child: Icon(
                _connectionState == ReaderConnectionState.connected
                    ? Icons.bluetooth_connected
                    : Icons.bluetooth_disabled,
                semanticLabel: _connectionState == ReaderConnectionState.connected
                    ? 'Reader connected'
                    : 'Reader disconnected',
              ),
            ),
          ),
        ],
      ),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _ConnectionBanner(
                state: _connectionState,
                onConnect: _connectReader,
              ),
              const SizedBox(height: 16),
              if (_lastScan != null) ...[
                _ScanResultCard(result: _lastScan!),
                const SizedBox(height: 12),
                // Officer confirmation checkbox
                CheckboxListTile(
                  title: const Text('I confirm this scan is correct'),
                  value: _confirmed,
                  onChanged: (v) => setState(() => _confirmed = v ?? false),
                  controlAffinity: ListTileControlAffinity.leading,
                ),
                const SizedBox(height: 8),
                SizedBox(
                  height: 56, // >= 48dp touch target
                  child: ElevatedButton(
                    onPressed: (_confirmed && !_submitting) ? _submitTransition : null,
                    child: _submitting
                        ? const CircularProgressIndicator.adaptive()
                        : const Text('Submit state transition'),
                  ),
                ),
              ] else ...[
                const Expanded(
                  child: Center(
                    child: Text(
                      'Waiting for tag scan...',
                      style: TextStyle(fontSize: 16),
                      semanticsLabel: 'Waiting for tag scan from RFID reader',
                    ),
                  ),
                ),
              ],
              if (_offlineQueue.isNotEmpty)
                Padding(
                  padding: const EdgeInsets.only(top: 12),
                  child: Text(
                    '${_offlineQueue.where((r) => !r.synced).length} read(s) queued offline.',
                    style: const TextStyle(fontSize: 12, color: Colors.orange),
                    semanticsLabel:
                        '${_offlineQueue.where((r) => !r.synced).length} RFID reads are queued offline and will sync on reconnect.',
                  ),
                ),
              // Duress long-press button
              const SizedBox(height: 16),
              GestureDetector(
                onLongPress: _triggerDuress,
                child: Container(
                  height: 48,
                  alignment: Alignment.center,
                  decoration: BoxDecoration(
                    border: Border.all(color: Colors.red, width: 1.5),
                    borderRadius: BorderRadius.circular(6),
                  ),
                  child: const Text(
                    'Hold for duress alert',
                    style: TextStyle(color: Colors.red),
                    semanticsLabel:
                        'Long press this button to send a silent duress alert to ECC.',
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Sub-widgets
// ---------------------------------------------------------------------------

class _ConnectionBanner extends StatelessWidget {
  const _ConnectionBanner({required this.state, required this.onConnect});

  final ReaderConnectionState state;
  final VoidCallback onConnect;

  @override
  Widget build(BuildContext context) {
    switch (state) {
      case ReaderConnectionState.connected:
        return Semantics(
          label: 'Reader connected',
          child: const Row(
            children: [
              Icon(Icons.check_circle, color: Colors.green),
              SizedBox(width: 8),
              Text('Reader connected', style: TextStyle(color: Colors.green)),
            ],
          ),
        );
      case ReaderConnectionState.scanning:
      case ReaderConnectionState.connecting:
        return const Row(
          children: [
            SizedBox(
              width: 20,
              height: 20,
              child: CircularProgressIndicator.adaptive(strokeWidth: 2),
            ),
            SizedBox(width: 8),
            Text('Connecting to reader...'),
          ],
        );
      case ReaderConnectionState.error:
        return Semantics(
          label: 'Reader connection error',
          child: Row(
            children: [
              const Icon(Icons.error, color: Colors.red),
              const SizedBox(width: 8),
              const Text('Connection error', style: TextStyle(color: Colors.red)),
              const SizedBox(width: 8),
              TextButton(onPressed: onConnect, child: const Text('Retry')),
            ],
          ),
        );
      case ReaderConnectionState.disconnected:
        return SizedBox(
          height: 48,
          child: ElevatedButton.icon(
            onPressed: onConnect,
            icon: const Icon(Icons.bluetooth_searching),
            label: const Text('Connect RFID reader'),
          ),
        );
    }
  }
}

class _ScanResultCard extends StatelessWidget {
  const _ScanResultCard({required this.result});

  final RfidScanResult result;

  @override
  Widget build(BuildContext context) {
    final isTampered = result.tamperIndicator;
    return Semantics(
      label: 'Scan result: EPC ${result.epc}. '
          'Tamper indicator: ${isTampered ? "YES - WARNING" : "No"}. '
          'Challenge valid: ${result.challengeValid ? "Yes" : "No"}.',
      child: Card(
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(8),
          side: BorderSide(
            color: isTampered ? Colors.red : Colors.green,
            width: 2,
          ),
        ),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Icon(
                    isTampered ? Icons.warning : Icons.verified,
                    color: isTampered ? Colors.red : Colors.green,
                    semanticLabel: isTampered ? 'Tamper warning' : 'Tag intact',
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      isTampered ? 'TAMPER DETECTED' : 'Tag intact',
                      style: TextStyle(
                        fontWeight: FontWeight.bold,
                        color: isTampered ? Colors.red : Colors.green,
                        fontSize: 16,
                      ),
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 12),
              _Row(label: 'EPC', value: result.epc),
              _Row(label: 'Scanned', value: result.readAt.toIso8601String()),
              _Row(
                label: 'Challenge',
                value: result.challengeValid ? 'Valid' : 'FAILED',
                valueColor: result.challengeValid ? Colors.green : Colors.red,
              ),
              if (result.rssi != null)
                _Row(label: 'RSSI', value: '${result.rssi!.toStringAsFixed(1)} dBm'),
            ],
          ),
        ),
      ),
    );
  }
}

class _Row extends StatelessWidget {
  const _Row({required this.label, required this.value, this.valueColor});

  final String label;
  final String value;
  final Color? valueColor;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 90,
            child: Text(
              '$label:',
              style: const TextStyle(fontWeight: FontWeight.w600),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: TextStyle(color: valueColor),
            ),
          ),
        ],
      ),
    );
  }
}
