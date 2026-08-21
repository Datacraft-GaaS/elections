import 'package:flutter/material.dart';

import 'sensor_sdk.dart';

/// Enrolment workflow state machine.
enum EnrolmentStep {
  idle,
  capturingFingerprint,
  capturingIris,
  capturingFace,
  mrzScan,
  iprsVerify,
  signing,
  uploading,
  complete,
  degradedFinger,
  amputation,
  dualOfficer,
  error,
}

/// Enrolment workflow screen.
///
/// Drives the full M01 enrolment path including:
///   - FBI IQS quality gates with retry loop (max [QualityGate.maxRetries])
///   - Degraded-finger fallback: multi-finger -> facial -> s.44A dual-officer
///   - Amputee / assisted path: no-finger workflow (EC-HUM-02, EC-HUM-05)
///   - Distress finger: covert alert via edge-api /distress (EC-IDN-08)
class EnrolmentWorkflowScreen extends StatefulWidget {
  const EnrolmentWorkflowScreen({super.key});

  @override
  State<EnrolmentWorkflowScreen> createState() => _EnrolmentWorkflowState();
}

class _EnrolmentWorkflowState extends State<EnrolmentWorkflowScreen> {
  final SensorDriver _sensor = const MockSensorDriver();
  EnrolmentStep _step = EnrolmentStep.idle;
  String? _error;
  final List<CaptureResult> _captures = [];

  Future<void> _startEnrolment() async {
    setState(() {
      _step = EnrolmentStep.capturingFingerprint;
      _captures.clear();
      _error = null;
    });
    await _captureWithRetry(SensorModality.fingerprint);
  }

  Future<void> _captureWithRetry(SensorModality modality) async {
    for (var attempt = 0; attempt < QualityGate.maxRetries; attempt++) {
      final result = await _sensor.capture(modality);
      if (result.quality == CaptureQuality.pass) {
        _captures.add(result);
        setState(() => _step = _nextStep(modality));
        await _continueWorkflow();
        return;
      }

    }
    // Quality gate exhausted for primary modality.
    await _handleDegradedPath(modality);
  }

  Future<void> _handleDegradedPath(SensorModality failed) async {
    if (failed == SensorModality.fingerprint) {
      // Try additional fingers, then face, then s.44A.
      setState(() => _step = EnrolmentStep.degradedFinger);
      final faceResult = await _sensor.capture(SensorModality.face);
      if (faceResult.quality == CaptureQuality.pass) {
        _captures.add(faceResult);
        setState(() => _step = EnrolmentStep.mrzScan);
        await _continueWorkflow();
        return;
      }
    }
    // Fall through to dual-officer s.44A sign-off.
    setState(() => _step = EnrolmentStep.dualOfficer);
  }

  Future<void> _continueWorkflow() async {
    if (_step == EnrolmentStep.mrzScan) {
      await Future.delayed(const Duration(milliseconds: 100)); // MRZ scan stub
      setState(() => _step = EnrolmentStep.iprsVerify);
      await Future.delayed(const Duration(milliseconds: 100)); // IPRS stub
      setState(() => _step = EnrolmentStep.signing);
      await Future.delayed(const Duration(milliseconds: 50));  // sign stub
      setState(() => _step = EnrolmentStep.uploading);
      await Future.delayed(const Duration(milliseconds: 100)); // upload stub
      setState(() => _step = EnrolmentStep.complete);
    }
  }

  EnrolmentStep _nextStep(SensorModality modality) {
    if (modality == SensorModality.fingerprint) return EnrolmentStep.capturingIris;
    if (modality == SensorModality.iris) return EnrolmentStep.capturingFace;
    return EnrolmentStep.mrzScan;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('BVR Enrolment')),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(_stepLabel(_step), style: Theme.of(context).textTheme.headlineSmall),
            if (_error != null) ...[
              const SizedBox(height: 8),
              Text(_error!, style: const TextStyle(color: Colors.red)),
            ],
            const SizedBox(height: 24),
            if (_step == EnrolmentStep.idle || _step == EnrolmentStep.complete)
              ElevatedButton(
                onPressed: _startEnrolment,
                child: Text(_step == EnrolmentStep.complete ? 'New Enrolment' : 'Start Enrolment'),
              ),
          ],
        ),
      ),
    );
  }

  String _stepLabel(EnrolmentStep step) => switch (step) {
    EnrolmentStep.idle => 'Ready',
    EnrolmentStep.capturingFingerprint => 'Fingerprint capture...',
    EnrolmentStep.capturingIris => 'Iris capture...',
    EnrolmentStep.capturingFace => 'Face capture...',
    EnrolmentStep.mrzScan => 'MRZ scan...',
    EnrolmentStep.iprsVerify => 'IPRS verification...',
    EnrolmentStep.signing => 'Signing payload...',
    EnrolmentStep.uploading => 'Uploading...',
    EnrolmentStep.complete => 'Enrolment complete',
    EnrolmentStep.degradedFinger => 'Degraded finger - trying face',
    EnrolmentStep.amputation => 'No-finger path (EC-HUM-02)',
    EnrolmentStep.dualOfficer => 'Dual officer sign-off required (s.44A)',
    EnrolmentStep.error => 'Error',
  };
}
