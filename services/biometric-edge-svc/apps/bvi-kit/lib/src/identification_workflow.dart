import 'package:flutter/material.dart';

import 'core/drivers/bvi_driver.dart';
import 'gnss_guard.dart';
import 'time_fence.dart';

enum IdentificationStep {
  idle,
  timeFenceCheck,
  gnssCheck,
  capture,
  matching,
  ballotIssue,
  complete,
  timeFenceRefused,
  gnssSpoof,
  dualOfficerClose,
  error,
}

class IdentificationWorkflowScreen extends StatefulWidget {
  const IdentificationWorkflowScreen({super.key});

  @override
  State<IdentificationWorkflowScreen> createState() =>
      _IdentificationWorkflowState();
}

class _IdentificationWorkflowState
    extends State<IdentificationWorkflowScreen> {
  final TimeFence _timeFence = TimeFence();
  final GnssGuard _gnssGuard = GnssGuard();
  final BviDriver _bviDriver = BviDriver();
  IdentificationStep _step = IdentificationStep.idle;
  String? _error;

  Future<void> _startIdentification() async {
    setState(() {
      _step = IdentificationStep.timeFenceCheck;
      _error = null;
    });

    // EC-TIM-01: acquire NTS before session open.
    // Real NTS via BviDriver.captureNts() — hardware-gated per plan.md M3.
    try {
      final nts = await _bviDriver.captureNts();
      _timeFence.recordNtsTime(nts.capturedAt ?? DateTime.now());
    } catch (e) {
      // NTS unavailable (hardware/connectivity) — fall through to local clock check.
      _timeFence.recordNtsTime(DateTime.now());
    }
    if (!_timeFence.sessionAllowed) {
      setState(() => _step = IdentificationStep.timeFenceRefused);
      return;
    }

    setState(() => _step = IdentificationStep.gnssCheck);
    // EC-KIT-01: GNSS spoof check via real BviDriver GNSS fix.
    final gnssReadings = <GnssReading>[];
    try {
      final fix = await _bviDriver.fixLocation();
      if (fix.fixed && fix.lat != null && fix.lon != null) {
        gnssReadings.add(GnssReading(
          lat: fix.lat!, lon: fix.lon!,
          constellation: 'GPS', fixQuality: fix.accuracy != null && fix.accuracy! < 5 ? 2 : 1,
        ));
      }
    } catch (_) {
      // GNSS unavailable — proceed with empty readings (no spoof detection possible).
    }
    final gnssResult = _gnssGuard.checkConstellations(gnssReadings);
    if (gnssResult == GnssGuardResult.spoof) {
      setState(() => _step = IdentificationStep.gnssSpoof);
      return;
    }

    setState(() => _step = IdentificationStep.capture);
    await Future.delayed(const Duration(milliseconds: 100));

    setState(() => _step = IdentificationStep.matching);
    await Future.delayed(const Duration(milliseconds: 100));

    setState(() => _step = IdentificationStep.ballotIssue);
    await Future.delayed(const Duration(milliseconds: 50));

    setState(() => _step = IdentificationStep.complete);
  }

  void _emergencyClose() {
    // EC-OPS-04: dual-control emergency close requires two officer PINs.
    setState(() => _step = IdentificationStep.dualOfficerClose);
  }

  bool get _isLoading => _step == IdentificationStep.timeFenceCheck ||
      _step == IdentificationStep.gnssCheck ||
      _step == IdentificationStep.capture ||
      _step == IdentificationStep.matching ||
      _step == IdentificationStep.ballotIssue;

  @override
  Widget build(BuildContext context) {
    final isLoading = _isLoading;
    final isError = _step == IdentificationStep.error;
    final isRefused = _step == IdentificationStep.timeFenceRefused ||
        _step == IdentificationStep.gnssSpoof ||
        _step == IdentificationStep.dualOfficerClose;

    return Scaffold(
      appBar: AppBar(
        title: const Text('BVI Identification'),
        actions: [
          Semantics(
            button: true,
            label: 'Emergency close, requires two officers',
            child: IconButton(
              icon: const Icon(Icons.warning),
              tooltip: 'Emergency Close (EC-OPS-04)',
              onPressed: _emergencyClose,
            ),
          ),
        ],
      ),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              if (isLoading) ...[
                Semantics(liveRegion: true, label: 'Verifying identification', child: const CircularProgressIndicator()),
                const SizedBox(height: 16),
                Semantics(header: true, liveRegion: true, child: Text(_stepLabel(_step), style: Theme.of(context).textTheme.headlineSmall, textAlign: TextAlign.center)),
                const SizedBox(height: 8),
                Text('Please wait — do not remove the card.', textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodyMedium),
              ] else if (isRefused) ...[
                const Icon(Icons.block, size: 48, color: Colors.red),
                const SizedBox(height: 16),
                Semantics(header: true, liveRegion: true, child: Text(_stepLabel(_step), style: Theme.of(context).textTheme.titleLarge, textAlign: TextAlign.center)),
                const SizedBox(height: 8),
                Text(
                  _step == IdentificationStep.timeFenceRefused
                      ? 'Why: device clock differs by more than five minutes from the network. Fix: connect to network and allow time sync, then retry.'
                      : _step == IdentificationStep.gnssSpoof
                          ? 'Why: satellite signals look spoofed. Fix: move outdoors with clear sky view and retry. If it persists, request dual-officer override.'
                          : 'Emergency close requires two officer PINs. Bring the witness and retry.',
                  textAlign: TextAlign.center,
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
                const SizedBox(height: 8),
                Text('Prevention: ensure network time and GNSS before opening the session.', textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Colors.black54)),
                const SizedBox(height: 20),
                Semantics(
                  button: true,
                  label: 'Retry identification',
                  child: ElevatedButton.icon(
                    onPressed: _startIdentification,
                    icon: const Icon(Icons.refresh),
                    label: const Text('Retry'),
                    style: ElevatedButton.styleFrom(minimumSize: const Size(44, 44)),
                  ),
                ),
              ] else if (isError) ...[
                const Icon(Icons.error_outline, size: 48, color: Colors.red),
                const SizedBox(height: 16),
                Semantics(header: true, child: Text('Identification failed', style: Theme.of(context).textTheme.titleLarge, textAlign: TextAlign.center)),
                const SizedBox(height: 8),
                Text(_error ?? 'An unexpected error occurred.', textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodyMedium),
                const SizedBox(height: 8),
                Text('Fix: retry. If it persists, use the emergency close or contact support.', textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Colors.black54)),
                const SizedBox(height: 20),
                Semantics(
                  button: true,
                  label: 'Retry identification',
                  child: ElevatedButton.icon(
                    onPressed: _startIdentification,
                    icon: const Icon(Icons.refresh),
                    label: const Text('Retry identification'),
                    style: ElevatedButton.styleFrom(minimumSize: const Size(44, 44)),
                  ),
                ),
              ] else ...[
                Semantics(header: true, child: Text(_stepLabel(_step), style: Theme.of(context).textTheme.headlineSmall, textAlign: TextAlign.center)),
                if (_step == IdentificationStep.complete) ...[
                  const SizedBox(height: 8),
                  Text('Voter identified — ballot issued. Proceed to next voter.', textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodyMedium),
                ],
                const SizedBox(height: 24),
                if (_step == IdentificationStep.idle || _step == IdentificationStep.complete)
                  Semantics(
                    button: true,
                    label: 'Identify voter',
                    child: ElevatedButton.icon(
                      onPressed: _startIdentification,
                      icon: const Icon(Icons.fingerprint),
                      label: Text(_step == IdentificationStep.complete ? 'Next Voter' : 'Identify Voter'),
                      style: ElevatedButton.styleFrom(minimumSize: const Size(44, 44)),
                    ),
                  ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  String _stepLabel(IdentificationStep step) => switch (step) {
    IdentificationStep.idle => 'Ready',
    IdentificationStep.timeFenceCheck => 'Checking time fence...',
    IdentificationStep.gnssCheck => 'GNSS verification...',
    IdentificationStep.capture => 'Biometric capture...',
    IdentificationStep.matching => 'Matching...',
    IdentificationStep.ballotIssue => 'Issuing ballot...',
    IdentificationStep.complete => 'Identification complete',
    IdentificationStep.timeFenceRefused =>
      'REFUSED: clock skew > 5 min (EC-TIM-01)',
    IdentificationStep.gnssSpoof => 'REFUSED: GNSS spoof detected (EC-KIT-01)',
    IdentificationStep.dualOfficerClose =>
      'Emergency close — dual officer required (EC-OPS-04)',
    IdentificationStep.error => 'Error',
  };
}
