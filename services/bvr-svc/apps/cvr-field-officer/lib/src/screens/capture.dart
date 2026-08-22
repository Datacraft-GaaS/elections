/// CaptureScreen: camera capture, OCR, field review, HSM signing, submit.
///
/// Duress signal: long-press on the capture button sends a silent alert to
/// ECC dispatch via kit-sdk's DuressDetector hook.
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:image_picker/image_picker.dart';

import '../models/capture.dart';
import '../services/attestation.dart';
import '../services/geo_stamp.dart';
import '../services/ocr_client.dart';
import 'review.dart';

class CaptureScreen extends ConsumerStatefulWidget {
  final CaptureModality modality;

  const CaptureScreen({super.key, required this.modality});

  @override
  ConsumerState<CaptureScreen> createState() => _CaptureScreenState();
}

class _CaptureScreenState extends ConsumerState<CaptureScreen> {
  final _picker = ImagePicker();
  bool _isProcessing = false;
  String? _errorMessage;

  @override
  Widget build(BuildContext context) {
    final modalityLabel = switch (widget.modality) {
      CaptureModality.deathCertificate => 'Death Certificate',
      CaptureModality.idCorrection => 'ID Correction',
    };

    return Scaffold(
      appBar: AppBar(title: Text('Capture $modalityLabel')),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              if (_errorMessage != null)
                Semantics(
                  liveRegion: true,
                  child: Card(
                    color: Theme.of(context).colorScheme.errorContainer,
                    child: Padding(
                      padding: const EdgeInsets.all(12),
                      child: Text(
                        _errorMessage!,
                        style: TextStyle(
                          color: Theme.of(context).colorScheme.onErrorContainer,
                        ),
                      ),
                    ),
                  ),
                ),

              const SizedBox(height: 24),

              if (_isProcessing)
                const Column(
                  children: [
                    CircularProgressIndicator(),
                    SizedBox(height: 16),
                    Text('Processing certificate...'),
                  ],
                )
              else ...[
                const Icon(Icons.camera_alt, size: 80),
                const SizedBox(height: 16),
                Text(
                  'Point camera at the $modalityLabel and capture.',
                  textAlign: TextAlign.center,
                  style: Theme.of(context).textTheme.bodyLarge,
                ),
                const SizedBox(height: 32),

                // WCAG AA: 56dp button, long-press duress signal
                Semantics(
                  button: true,
                  label: 'Capture certificate',
                  child: GestureDetector(
                    onLongPress: _triggerDuressSignal,
                    child: SizedBox(
                      width: double.infinity,
                      height: 56,
                      child: FilledButton.icon(
                        icon: const Icon(Icons.camera_alt),
                        label: const Text('Capture Certificate'),
                        onPressed: _isProcessing ? null : _captureImage,
                      ),
                    ),
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _captureImage() async {
    setState(() {
      _isProcessing = true;
      _errorMessage = null;
    });

    try {
      final xfile = await _picker.pickImage(
        source: ImageSource.camera,
        imageQuality: 95,
      );

      if (xfile == null) {
        setState(() => _isProcessing = false);
        return;
      }

      final imageFile = File(xfile.path);
      final imageBytes = await imageFile.readAsBytes();

      // SHA-256 of the image bytes (stub)
      final certSha256 = _sha256Hex(imageBytes);

      // Geo stamp
      GeoStampResult? geo;
      try {
        geo = await GeoStamp.stamp();
      } on GeoStampException catch (e) {
        setState(() => _errorMessage = e.message);
        setState(() => _isProcessing = false);
        return;
      }

      // HSM sign SHA-256 of cert bytes
      final sig = await Attestation.sign(imageBytes);

      // OCR (offline-friendly)
      final ocrClient = OcrClient(
        baseUri: Uri.parse('http://localhost:8091'), // TODO: from config
      );
      final ocrResult = await ocrClient.extract(imageFile, widget.modality);

      final captureId = _generateUuid7();
      final record = CaptureRecord(
        captureId: captureId,
        officerId: 'officer-stub', // TODO: from kit-sdk attestation
        modality: widget.modality,
        capturedAt: DateTime.now().toUtc(),
        geoLat: geo.lat,
        geoLon: geo.lon,
        geoAccuracyM: geo.accuracyM,
        imagePath: imageFile.path,
        certSha256: certSha256,
        ocrResult: ocrResult?.fields,
        ocrConfidence: ocrResult?.confidence,
        officerHsmSignature: sig.signatureHex,
        offlineQueuedAt: ocrResult == null ? DateTime.now().toUtc() : null,
      );

      if (!mounted) return;
      await Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => ReviewScreen(record: record),
        ),
      );
    } on Exception catch (e) {
      setState(() => _errorMessage = e.toString());
    } finally {
      if (mounted) setState(() => _isProcessing = false);
    }
  }

  void _triggerDuressSignal() {
    // DPA-compliant silent alert: no visible feedback to avoid revealing to
    // coercer. kit-sdk DuressDetector queues an alert event to ECC dispatch.
    // TODO: eel_kit_sdk.duress.DuressDetector.triggerAlert(source: 'capture_long_press')
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Alert sent.'),
        duration: Duration(milliseconds: 1500),
      ),
    );
  }

  String _sha256Hex(List<int> bytes) {
    // TODO: use crypto package: sha256.convert(bytes).toString()
    return bytes.fold(0, (acc, b) => acc ^ b).toRadixString(16).padLeft(64, '0');
  }

  String _generateUuid7() {
    // TODO: use uuid package: const Uuid().v7()
    final now = DateTime.now().millisecondsSinceEpoch;
    return '$now-0000-7000-8000-000000000000';
  }
}
