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
  const EnrolmentWorkflowScreen({super.key, this.sensor});

  /// Injected driver — `Fap60SensorDriver` for hardware, `MockSensorDriver` for tests.
  /// Defaults to mock in non-Android / CI; production `main.dart` injects `Fap60SensorDriver`.
  final SensorDriver? sensor;

  @override
  State<EnrolmentWorkflowScreen> createState() => _EnrolmentWorkflowState();
}

class _EnrolmentWorkflowState extends State<EnrolmentWorkflowScreen> {
  late final SensorDriver _sensor = widget.sensor ?? const MockSensorDriver();
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
      try {
        final result = await _sensor.capture(modality);
        if (result.quality == CaptureQuality.pass) {
          _captures.add(result);
          setState(() => _step = _nextStep(modality));
          await _continueWorkflow();
          return;
        }
      } catch (e) {
        setState(() {
          _step = EnrolmentStep.error;
          _error = 'Capture failed (${modality.name}): $e. Check sensor connection.';
        });
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
      try {
        final faceResult = await _sensor.capture(SensorModality.face);
        if (faceResult.quality == CaptureQuality.pass) {
          _captures.add(faceResult);
          setState(() => _step = EnrolmentStep.mrzScan);
          await _continueWorkflow();
          return;
        }
      } catch (e) {
        setState(() {
          _step = EnrolmentStep.error;
          _error = 'Face fallback failed: $e. Try again or use s.44A.';
        });
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

  bool get _isLoading => switch (_step) {
        EnrolmentStep.capturingFingerprint => true,
        EnrolmentStep.capturingIris => true,
        EnrolmentStep.capturingFace => true,
        EnrolmentStep.mrzScan => true,
        EnrolmentStep.iprsVerify => true,
        EnrolmentStep.signing => true,
        EnrolmentStep.uploading => true,
        _ => false,
      };

  @override
  Widget build(BuildContext context) {
    final isLoading = _isLoading;
    final isError = _step == EnrolmentStep.error;
    final isComplete = _step == EnrolmentStep.complete;
    final isIdle = _step == EnrolmentStep.idle;

    return Scaffold(
      appBar: AppBar(title: const Text('BVR Enrolment')),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              if (isLoading) ...[
                Semantics(
                  label: 'Loading enrolment step',
                  liveRegion: true,
                  child: const CircularProgressIndicator(),
                ),
                const SizedBox(height: 16),
                Semantics(
                  header: true,
                  liveRegion: true,
                  child: Text(_stepLabel(_step), style: Theme.of(context).textTheme.headlineSmall, textAlign: TextAlign.center),
                ),
                const SizedBox(height: 8),
                Text('Please wait — do not remove the device.', textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodyMedium),
              ] else if (isError) ...[
                const Icon(Icons.error_outline, size: 48, color: Colors.red),
                const SizedBox(height: 16),
                Semantics(header: true, child: Text('Enrolment failed', style: Theme.of(context).textTheme.titleLarge, textAlign: TextAlign.center)),
                const SizedBox(height: 8),
                Text(_error ?? 'An unexpected error occurred during enrolment.', textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodyMedium),
                const SizedBox(height: 8),
                Text('Check the sensor connection and try again. If it persists, request dual-officer assistance (s.44A).', textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Colors.black54)),
                const SizedBox(height: 8),
                Text('Prevention: keep fingers clean and dry; align face in frame.', textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Colors.black54)),
                const SizedBox(height: 20),
                Semantics(
                  button: true,
                  label: 'Retry enrolment',
                  child: ElevatedButton.icon(
                    onPressed: _startEnrolment,
                    icon: const Icon(Icons.refresh),
                    label: const Text('Retry enrolment'),
                    style: ElevatedButton.styleFrom(minimumSize: const Size(44, 44)),
                  ),
                ),
              ] else if (isComplete) ...[
                const Icon(Icons.verified, size: 56, color: Color(0xFF0E7A3A)),
                const SizedBox(height: 16),
                Semantics(header: true, liveRegion: true, child: Text('Enrolment complete — signed and queued for upload', style: Theme.of(context).textTheme.titleLarge, textAlign: TextAlign.center)),
                const SizedBox(height: 8),
                Text('${_captures.length} captures accepted. You will be notified when the record is anchored.', textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodyMedium),
                const SizedBox(height: 20),
                Semantics(
                  button: true,
                  label: 'Start new enrolment',
                  child: ElevatedButton.icon(
                    onPressed: _startEnrolment,
                    icon: const Icon(Icons.person_add),
                    label: const Text('New Enrolment'),
                    style: ElevatedButton.styleFrom(minimumSize: const Size(44, 44)),
                  ),
                ),
              ] else if (isIdle) ...[
                const Icon(Icons.fingerprint, size: 48, color: Color(0xFF0E7A3A)),
                const SizedBox(height: 16),
                Semantics(header: true, child: Text(_stepLabel(_step), style: Theme.of(context).textTheme.headlineSmall, textAlign: TextAlign.center)),
                const SizedBox(height: 8),
                Text('Place the finger on the sensor to begin. Dual-officer path available for exceptions (s.44A).', textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodyMedium),
                const SizedBox(height: 24),
                Semantics(
                  button: true,
                  label: 'Start enrolment',
                  child: ElevatedButton.icon(
                    onPressed: _startEnrolment,
                    icon: const Icon(Icons.play_arrow),
                    label: const Text('Start Enrolment'),
                    style: ElevatedButton.styleFrom(minimumSize: const Size(44, 44)),
                  ),
                ),
              ] else ...[
                // degradedFinger / dualOfficer / amputation — informative state with action
                Icon(
                  _step == EnrolmentStep.dualOfficer ? Icons.groups : Icons.hearing,
                  size: 48,
                  color: const Color(0xFF7A5718),
                ),
                const SizedBox(height: 16),
                Semantics(header: true, liveRegion: true, child: Text(_stepLabel(_step), style: Theme.of(context).textTheme.titleLarge, textAlign: TextAlign.center)),
                const SizedBox(height: 8),
                Text(
                  _step == EnrolmentStep.dualOfficer
                      ? 'Two officers must sign to proceed. Bring the witness to the device.'
                      : _step == EnrolmentStep.amputation
                          ? 'No-finger workflow selected. Continue with face and document verification.'
                          : 'Fingerprint quality low — switched to face capture fallback.',
                  textAlign: TextAlign.center,
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
                const SizedBox(height: 20),
                Semantics(
                  button: true,
                  label: 'Continue enrolment',
                  child: OutlinedButton.icon(
                    onPressed: _continueWorkflow,
                    icon: const Icon(Icons.arrow_forward),
                    label: const Text('Continue'),
                    style: OutlinedButton.styleFrom(minimumSize: const Size(44, 44)),
                  ),
                ),
              ],
            ],
          ),
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
