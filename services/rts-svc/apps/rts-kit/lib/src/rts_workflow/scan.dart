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

/// Detect form-corner candidates using a Sobel edge response count.
/// Returns the number of edge clusters found (expect ~4 for a flat form).
int _detectCorners(img.Image gray) {
  final points = _edgePoints(gray);
  if (points.isEmpty) return 0;
  // Cluster edge pixels on a coarse grid; distinct clusters approximate
  // candidate corners (4 for a well-framed flat form).
  final cells = <String>{};
  const cell = 16;
  for (final pt in points) {
    cells.add('x${pt.x ~/ cell}y${pt.y ~/ cell}');
  }
  return cells.length.clamp(0, 8);
}

/// Downscaled Sobel edge pixels (coordinates in FULL-RESOLUTION space).
List<({int x, int y})> _edgePoints(img.Image gray) {
  const probeWidth = 200;
  final scaled = img.copyResize(gray, width: probeWidth);
  final scale = gray.width / probeWidth;
  final edges = img.sobel(scaled);
  final pts = <({int x, int y})>[];
  for (final px in edges) {
    if (img.getLuminance(px) > 200) {
      pts.add((x: (px.x * scale).round(), y: (px.y * scale).round()));
    }
  }
  return pts;
}

/// Locate the four document corners from edge pixels using diagonal extremes:
/// TL minimises x+y, TR maximises x-y, BR maximises x+y, BL minimises x-y.
/// Returns [tlX, tlY, trX, trY, brX, brY, blX, blY] or null when too few
/// edge pixels exist to bracket the page.
List<int>? _formCornersQuad(img.Image gray) {
  final pts = _edgePoints(gray);
  if (pts.length < 8) return null;

  ({int x, int y}) extreme(bool Function(({int x, int y}) a, ({int x, int y}) b) better) {
    var acc = pts.first;
    for (final p in pts.skip(1)) {
      if (better(p, acc)) acc = p;
    }
    return acc;
  }

  int score(({int x, int y}) p) => p.x + p.y;
  int diff(({int x, int y}) p) => p.x - p.y;

  final tl = extreme((a, b) => score(a) < score(b));
  final br = extreme((a, b) => score(a) > score(b));
  final tr = extreme((a, b) => diff(a) > diff(b));
  final bl = extreme((a, b) => diff(a) < diff(b));

  // Reject degenerate quads (corners collapsing onto each other).
  if ((tl.x - br.x).abs() < gray.width * 0.2 ||
      (tl.y - br.y).abs() < gray.height * 0.2) {
    return null;
  }
  return [tl.x, tl.y, tr.x, tr.y, br.x, br.y, bl.x, bl.y];
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

/// Apply four-point perspective correction.
///
/// Detects the form's corners via diagonal extremes over Sobel edges and
/// warps the quad to an upright rectangle with [img.copyRectify]. When the
/// quad cannot be trusted (too few edges / degenerate geometry) the function
/// falls back to levelled centre-crop so capture never blocks on corner
/// detection alone.
Uint8List applyPerspectiveCorrection(Uint8List rawJpeg) {
  final image = img.decodeImage(rawJpeg);
  if (image == null) throw StateError('Failed to decode image');

  final gray = img.grayscale(image);
  final quad = _formCornersQuad(gray);

  img.Image corrected;
  if (quad != null) {
    // copyRectify maps the source quad (tl, tr, br, bl) onto an upright rect.
    corrected = img.copyRectify(
      image,
      topLeft: img.Point(quad[0], quad[1]),
      topRight: img.Point(quad[2], quad[3]),
      bottomRight: img.Point(quad[4], quad[5]),
      bottomLeft: img.Point(quad[6], quad[7]),
    );
  } else {
    final cropped = img.copyCrop(
      image,
      x: (image.width * 0.05).round(),
      y: (image.height * 0.05).round(),
      width: (image.width * 0.90).round(),
      height: (image.height * 0.90).round(),
    );
    corrected = cropped;
  }

  // Normalise exposure: slight contrast lift compensates shadowed booths.
  final levelled = img.adjustColor(corrected, brightness: 0, contrast: 1.1);
  return Uint8List.fromList(img.encodeJpg(levelled, quality: 90));
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
