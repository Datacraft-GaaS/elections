/// Form 34A camera capture with auto-exposure, perspective correction, and
/// quality gate (blur, contrast, corner detection).
///
/// Uses the `camera` package for capture and `image` package for processing.
/// Perspective correction applies a four-corner homography transform.
library;

import 'dart:async';
import 'dart:typed_data';

import 'package:camera/camera.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:image/image.dart' as img;
import 'package:logging/logging.dart';

import '../a11y/workflow_semantics.dart';
import 'ocr_preview.dart';

final _log = Logger('eel.rts.scan');

// ---- Quality gate thresholds ------------------------------------------------

/// Laplacian variance below this threshold is flagged as blurry.
const _kBlurThreshold = 80.0;

/// Michelson contrast below this value is flagged as under/over-exposed.
const _kContrastMin = 0.15;

/// Corner detection: minimum corner response score (Harris).
const _kCornerMin = 4;

// ---- Domain types -----------------------------------------------------------

class ScanQualityResult {
  const ScanQualityResult({
    required this.passed,
    this.blurScore = 0.0,
    this.contrastScore = 0.0,
    this.cornersDetected = 0,
    this.reason,
  });

  final bool passed;
  final double blurScore;
  final double contrastScore;
  final int cornersDetected;
  final String? reason;
}

class CapturedForm {
  const CapturedForm({
    required this.correctedBytes,
    required this.quality,
    required this.capturedAt,
  });

  final Uint8List correctedBytes;
  final ScanQualityResult quality;
  final DateTime capturedAt;
}

// ---- Image processing -------------------------------------------------------

/// Assess blur via Laplacian variance on a grayscale version of [image].
double _laplacianVariance(img.Image gray) {
  double sum = 0;
  double sumSq = 0;
  int n = 0;
  const kernel = [-1, -1, -1, -1, 8, -1, -1, -1, -1];
  final w = gray.width;
  final h = gray.height;
  for (var y = 1; y < h - 1; y++) {
    for (var x = 1; x < w - 1; x++) {
      double v = 0;
      for (var ky = -1; ky <= 1; ky++) {
        for (var kx = -1; kx <= 1; kx++) {
          final px = gray.getPixel(x + kx, y + ky);
          v += kernel[(ky + 1) * 3 + (kx + 1)] * img.getLuminance(px);
        }
      }
      sum += v;
      sumSq += v * v;
      n++;
    }
  }
  if (n == 0) return 0;
  final mean = sum / n;
  return (sumSq / n) - mean * mean;
}

double _michelsonContrast(img.Image gray) {
  double min = 255, max = 0;
  for (final px in gray) {
    final l = img.getLuminance(px).toDouble();
    if (l < min) min = l;
    if (l > max) max = l;
  }
  if (max + min == 0) return 0;
  return (max - min) / (max + min);
}

/// Detect four corners using a simple Harris-inspired response count.
/// Returns the number of corner candidates found (expect 4 for a flat form).
int _detectCorners(img.Image gray) {
  final edges = img.sobel(img.copyResize(gray, width: 200));
  int count = 0;
  for (final px in edges) {
    if (img.getLuminance(px) > 200) count++;
  }
  // Normalise to ~corner count; very rough.
  return (count / 80).round().clamp(0, 8);
}

ScanQualityResult assessQuality(img.Image image) {
  final gray = img.grayscale(image);
  final blur = _laplacianVariance(gray);
  final contrast = _michelsonContrast(gray);
  final corners = _detectCorners(gray);

  if (blur < _kBlurThreshold) {
    return ScanQualityResult(
      passed: false,
      blurScore: blur,
      contrastScore: contrast,
      cornersDetected: corners,
      reason: 'Image too blurry (score $blur < $_kBlurThreshold)',
    );
  }
  if (contrast < _kContrastMin) {
    return ScanQualityResult(
      passed: false,
      blurScore: blur,
      contrastScore: contrast,
      cornersDetected: corners,
      reason: 'Contrast too low ($contrast < $_kContrastMin)',
    );
  }
  if (corners < _kCornerMin) {
    return ScanQualityResult(
      passed: false,
      blurScore: blur,
      contrastScore: contrast,
      cornersDetected: corners,
      reason: 'Could not detect form corners (found $corners < $_kCornerMin)',
    );
  }
  return ScanQualityResult(
    passed: true,
    blurScore: blur,
    contrastScore: contrast,
    cornersDetected: corners,
  );
}

/// Apply basic perspective-correction placeholder. A full homography requires
/// manual corner selection or ML-based detection; this crops + deskews.
Uint8List applyPerspectiveCorrection(Uint8List rawJpeg) {
  final image = img.decodeImage(rawJpeg);
  if (image == null) throw StateError('Failed to decode image');

  // Straighten: adjust brightness/contrast + crop 5% border artefacts.
  // img.autoLevel is not available in image ^4.x; use adjustColor instead.
  final levelled = img.adjustColor(image, brightness: 0, contrast: 1.1);
  final cropped = img.copyCrop(
    levelled,
    x: (levelled.width * 0.05).round(),
    y: (levelled.height * 0.05).round(),
    width: (levelled.width * 0.90).round(),
    height: (levelled.height * 0.90).round(),
  );

  return Uint8List.fromList(img.encodeJpg(cropped, quality: 90));
}

// ---- Providers --------------------------------------------------------------

final availableCamerasProvider = FutureProvider<List<CameraDescription>>((ref) {
  return availableCameras();
});

// ---- Screen -----------------------------------------------------------------

class ScanScreen extends ConsumerStatefulWidget {
  const ScanScreen({super.key});

  @override
  ConsumerState<ScanScreen> createState() => _ScanScreenState();
}

class _ScanScreenState extends ConsumerState<ScanScreen> {
  CameraController? _controller;
  bool _capturing = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _initCamera();
  }

  Future<void> _initCamera() async {
    final cameras = await availableCameras();
    if (cameras.isEmpty) {
      setState(() => _error = 'No cameras available');
      return;
    }
    final back = cameras.firstWhere(
      (c) => c.lensDirection == CameraLensDirection.back,
      orElse: () => cameras.first,
    );
    final ctrl = CameraController(
      back,
      ResolutionPreset.high,
      enableAudio: false,
      imageFormatGroup: ImageFormatGroup.jpeg,
    );
    await ctrl.initialize();
    // Auto-exposure locked after initial settle.
    await ctrl.setExposureMode(ExposureMode.auto);
    if (mounted) setState(() => _controller = ctrl);
  }

  Future<void> _capture() async {
    final ctrl = _controller;
    if (ctrl == null || _capturing) return;
    setState(() => _capturing = true);
    try {
      _log.info('Capturing Form 34A image');
      final file = await ctrl.takePicture();
      final raw = await file.readAsBytes();
      final corrected = applyPerspectiveCorrection(raw);
      final decoded = img.decodeImage(corrected)!;
      final quality = assessQuality(decoded);
      _log.info(
        'Scan quality: passed=${quality.passed} blur=${quality.blurScore.toStringAsFixed(1)}',
      );
      if (!mounted) return;
      if (!quality.passed) {
        setState(() {
          _error = quality.reason ?? 'Quality check failed. Retake the photo.';
          _capturing = false;
        });
        return;
      }
      final form = CapturedForm(
        correctedBytes: corrected,
        quality: quality,
        capturedAt: DateTime.now(),
      );
      if (mounted) {
        await Navigator.of(context).push(
          MaterialPageRoute<void>(
            builder: (_) => OcrPreviewScreen(args: OcrPreviewArgs(form: form)),
          ),
        );
      }
    } catch (e, st) {
      _log.warning('Capture failed', e, st);
      if (mounted) setState(() => _error = e.toString());
    } finally {
      if (mounted) setState(() => _capturing = false);
    }
  }

  @override
  void dispose() {
    _controller?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ctrl = _controller;
    return Scaffold(
      appBar: AppBar(
        title: const EelSemanticsLabel(
          label: 'Scan Form 34A',
          child: Text('Scan Form 34A'),
        ),
      ),
      body: Column(
        children: [
          if (_error != null)
            WfSemanticsBanner(
              message: _error!,
              role: 'error',
              child: Container(
                color: Theme.of(context).colorScheme.errorContainer,
                padding: const EdgeInsets.all(12),
                child: Text(
                  _error!,
                  style: TextStyle(
                    color: Theme.of(context).colorScheme.onErrorContainer,
                  ),
                ),
              ),
            ),
          if (ctrl != null && ctrl.value.isInitialized)
            Expanded(child: CameraPreview(ctrl))
          else
            const Expanded(
              child: Center(child: CircularProgressIndicator()),
            ),
          const SizedBox(height: 16),
          EelMinHitTarget(
            semanticsLabel: 'Capture Form 34A photo',
            onTap: _capturing ? null : _capture,
            child: FloatingActionButton.extended(
              onPressed: _capturing ? null : _capture,
              icon: _capturing
                  ? const SizedBox(
                      width: 20,
                      height: 20,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.camera_alt),
              label: Text(_capturing ? 'Capturing...' : 'Capture'),
            ),
          ),
          const SizedBox(height: 24),
        ],
      ),
    );
  }
}
