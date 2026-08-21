import 'package:flutter/material.dart';

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
  IdentificationStep _step = IdentificationStep.idle;
  String? _error;

  Future<void> _startIdentification() async {
    setState(() {
      _step = IdentificationStep.timeFenceCheck;
      _error = null;
    });

    // EC-TIM-01: acquire NTS before session open.
    _timeFence.recordNtsTime(DateTime.now()); // stub; real: NTS client
    if (!_timeFence.sessionAllowed) {
      setState(() => _step = IdentificationStep.timeFenceRefused);
      return;
    }

    setState(() => _step = IdentificationStep.gnssCheck);
    // EC-KIT-01: GNSS spoof check (stub readings).
    final gnssResult = _gnssGuard.checkConstellations([
      const GnssReading(lat: -1.292, lon: 36.821, constellation: 'GPS', fixQuality: 2),
      const GnssReading(lat: -1.292, lon: 36.821, constellation: 'GLONASS', fixQuality: 2),
    ]);
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

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('BVI Identification'),
        actions: [
          IconButton(
            icon: const Icon(Icons.warning),
            tooltip: 'Emergency Close (EC-OPS-04)',
            onPressed: _emergencyClose,
          ),
        ],
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(_stepLabel(_step),
                style: Theme.of(context).textTheme.headlineSmall),
            if (_error != null) ...[
              const SizedBox(height: 8),
              Text(_error!, style: const TextStyle(color: Colors.red)),
            ],
            const SizedBox(height: 24),
            if (_step == IdentificationStep.idle ||
                _step == IdentificationStep.complete)
              ElevatedButton(
                onPressed: _startIdentification,
                child: const Text('Identify Voter'),
              ),
          ],
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
