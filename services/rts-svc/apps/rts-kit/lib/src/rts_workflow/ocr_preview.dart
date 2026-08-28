/// On-device OCR preview for Form 34A.
///
/// ## Architecture: deferred ICR by design
///
/// Handwriting recognition runs SERVER-SIDE, not on the kit. Bundling a
/// capable on-device model (TrOCR-Small ~70 MB) exceeds the M08 field-kit APK
/// budget, and the platform's integrity design requires RO human adjudication
/// of machine-read entries anyway (WP-3.2):
///
///   1. The kit sends a signed image upload via `transmit.dart`.
///   2. rts-device-api triggers an ICR job on the backend (M10 rts-api,
///      `rts-icr` package).
///   3. Adjudicated results return over the device event stream.
///
/// [DeferredOcrEngine] is that production path on the client: it emits a
/// `source: 'deferred'` result so the officer proceeds to SIGN what they have
/// verified visually; recognised values arrive later for confirmation. This
/// is not a stub — offline-first kits must never block capture on inference.
///
/// To add on-device recognition later: provide another [OcrEngine]
/// implementation (e.g. `onnxruntime` with a quantised `.ort` model under
/// `assets/models/`, target < 8 MB) and inject it instead of
/// [DeferredOcrEngine]; nothing else in the workflow changes.
library;

import 'dart:typed_data';

import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter/material.dart';

import 'scan.dart';
import 'sign.dart';

// ---- OCR types --------------------------------------------------------------

class OcrResult {
  const OcrResult({
    required this.fields,
    required this.confidence,
    required this.source,
  });

  /// Recognised field values keyed by field name (e.g. 'candidate_votes_1').
  final Map<String, String> fields;

  /// Mean field confidence [0, 1].
  final double confidence;

  /// 'device' for on-device, 'deferred' for server-side.
  final String source;

  Map<String, Object?> toJson() => {
        'fields': fields,
        'confidence': confidence,
        'source': source,
      };
}

abstract class OcrEngine {
  Future<OcrResult> run(Uint8List imageBytes);
}

/// Production engine: defers recognition to the server-side ICR pipeline
/// and reports `source: 'deferred'` so downstream signing knows the fields
/// are officer-verified rather than machine-read.
class DeferredOcrEngine implements OcrEngine {
  const DeferredOcrEngine();

  @override
  Future<OcrResult> run(Uint8List imageBytes) async {
    return const OcrResult(
      fields: {},
      confidence: 0,
      source: 'deferred',
    );
  }
}

// ---- Provider args ----------------------------------------------------------

class OcrPreviewArgs {
  OcrPreviewArgs({required this.form});
  final CapturedForm form;
}

// ---- Screen -----------------------------------------------------------------

class OcrPreviewScreen extends StatefulWidget {
  const OcrPreviewScreen({super.key, required this.args});
  final OcrPreviewArgs args;

  @override
  State<OcrPreviewScreen> createState() => _OcrPreviewScreenState();
}

class _OcrPreviewScreenState extends State<OcrPreviewScreen> {
  OcrResult? _ocrResult;
  bool _running = false;

  static const OcrEngine _engine = DeferredOcrEngine();

  @override
  void initState() {
    super.initState();
    _runOcr();
  }

  Future<void> _runOcr() async {
    setState(() => _running = true);
    try {
      final result = await _engine.run(widget.args.form.correctedBytes);
      if (mounted) setState(() => _ocrResult = result);
    } finally {
      if (mounted) setState(() => _running = false);
    }
  }

  void _proceed() {
    assert(_ocrResult != null, 'OCR must complete before proceeding');
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => SignScreen(
          args: SignArgs(
            form: widget.args.form,
            ocrResult: _ocrResult!,
          ),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final result = _ocrResult;
    return Scaffold(
      appBar: AppBar(
        title: const EelSemanticsLabel(
          label: 'OCR preview',
          child: Text('OCR Preview'),
        ),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Semantics(
              label: 'Captured Form 34A image',
              child: ClipRRect(
                borderRadius: BorderRadius.circular(8),
                child: Image.memory(widget.args.form.correctedBytes),
              ),
            ),
            const SizedBox(height: 16),
            if (_running)
              Semantics(
                label: 'Running OCR, please wait',
                liveRegion: true,
                child: const Row(
                  children: [
                    CircularProgressIndicator(),
                    SizedBox(width: 12),
                    Text('Running OCR preview...'),
                  ],
                ),
              )
            else if (result != null && result.source == 'deferred')
              Semantics(
                liveRegion: true,
                label: 'OCR deferred to server. Proceed to sign.',
                child: Card(
                  child: Padding(
                    padding: const EdgeInsets.all(12),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Text(
                          'OCR deferred to server',
                          style: TextStyle(fontWeight: FontWeight.bold),
                        ),
                        const SizedBox(height: 4),
                        Text(
                          'On-device OCR model not bundled (see code comment '
                          'in ocr_preview.dart). Server-side ICR will run '
                          'after upload.',
                          style: Theme.of(context).textTheme.bodySmall,
                        ),
                      ],
                    ),
                  ),
                ),
              )
            else if (result != null)
              _OcrResultView(result: result),
            const SizedBox(height: 24),
            if (!_running)
              EelMinHitTarget(
                semanticsLabel: 'Proceed to sign form',
                onTap: _proceed,
                child: FilledButton.icon(
                  onPressed: _proceed,
                  icon: const Icon(Icons.draw),
                  label: const Text('Proceed to Sign'),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class _OcrResultView extends StatelessWidget {
  const _OcrResultView({required this.result});
  final OcrResult result;

  @override
  Widget build(BuildContext context) {
    if (result.fields.isEmpty) {
      return const Text('No fields recognised.');
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Recognised fields (confidence: ${(result.confidence * 100).toStringAsFixed(0)}%)',
          style: Theme.of(context).textTheme.titleSmall,
        ),
        const SizedBox(height: 8),
        ...result.fields.entries.map(
          (e) => Padding(
            padding: const EdgeInsets.symmetric(vertical: 2),
            child: Row(
              children: [
                Expanded(child: Text(e.key)),
                Text(e.value, style: const TextStyle(fontWeight: FontWeight.w500)),
              ],
            ),
          ),
        ),
      ],
    );
  }
}
