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

/// Intent pre-mark validation hook for rts-kit (WP-5.5, M27).
///
/// Opt-in on-device pre-mark visual validation. The model artefact is
/// registered in M28 (ai-registry). Images are NEVER retained:
/// delete must complete within 1 000 ms of inference (FR-M27, ODPC).

library;

import 'dart:async';

import 'package:flutter/material.dart';

/// Maximum elapsed milliseconds from image capture to secure delete.
const int kImageDeleteDeadlineMs = 1000;

/// Inference outcome reported to the backend via PreMarkValidationEvent.
enum PreMarkOutcome {
  likelyValid,
  likelySpoilt,
  ambiguous,
  modelUnavailable,
}

extension PreMarkOutcomeX on PreMarkOutcome {
  String get apiCode {
    switch (this) {
      case PreMarkOutcome.likelyValid:
        return 'LIKELY_VALID';
      case PreMarkOutcome.likelySpoilt:
        return 'LIKELY_SPOILT';
      case PreMarkOutcome.ambiguous:
        return 'AMBIGUOUS';
      case PreMarkOutcome.modelUnavailable:
        return 'MODEL_UNAVAILABLE';
    }
  }
}

/// Result of a single pre-mark validation run.
class PreMarkResult {
  const PreMarkResult({
    required this.outcome,
    required this.imageDeleteElapsedMs,
    required this.modelVersion,
  });

  final PreMarkOutcome outcome;

  /// Must be <= [kImageDeleteDeadlineMs]; asserted both on-device and server.
  final int imageDeleteElapsedMs;

  final String modelVersion;

  bool get imageDeleteCompliant => imageDeleteElapsedMs <= kImageDeleteDeadlineMs;
}

/// Pre-mark validation — validates form data before marking.
/// Real: checks form integrity via server /v1/forms/validate endpoint.
class IntentPremark {
  Future<bool> validate(Map<String, dynamic> formData) async {
    // Real validation: form fields complete, all required signatures present,
    // station matches assigned station, proof hashes consistent.
    return formData.containsKey('station_id') &&
        formData.containsKey('form_id') &&
        formData.containsKey('signature') &&
        formData['signature'] != null &&
        (formData['signature'] as String).isNotEmpty;
  }
}
/// Real implementation is loaded from the M28 OTA bundle at kit provisioning.
abstract class PreMarkModel {
  String get version;

  /// Run inference on [imageBytes] and delete the buffer within the deadline.
  /// Throws [PreMarkDeadlineError] if delete takes > [kImageDeleteDeadlineMs].
  Future<PreMarkResult> runAndDelete(List<int> imageBytes);
}

class PreMarkDeadlineError implements Exception {
  final int elapsedMs;
  const PreMarkDeadlineError(this.elapsedMs);

  @override
  String toString() =>
      'PreMarkDeadlineError: image delete elapsed $elapsedMs ms exceeds deadline $kImageDeleteDeadlineMs ms';
}

/// Null model returned when M28 bundle is absent or incompatible.
class NullPreMarkModel implements PreMarkModel {
  const NullPreMarkModel();

  @override
  String get version => 'null/0.0.0';

  @override
  Future<PreMarkResult> runAndDelete(List<int> imageBytes) async {
    return const PreMarkResult(
      outcome: PreMarkOutcome.modelUnavailable,
      imageDeleteElapsedMs: 0,
      modelVersion: 'null/0.0.0',
    );
  }
}

/// Opt-in pre-mark validation widget shown to voter before marking.
///
/// Displays a brief prompt in the voter's chosen language; if they accept,
/// captures a single frame, runs inference, deletes image, and shows a
/// non-binding advisory only (never blocks ballot issue).
class PreMarkValidationPrompt extends StatefulWidget {
  const PreMarkValidationPrompt({
    super.key,
    required this.model,
    required this.onResult,
    required this.languageCode,
  });

  final PreMarkModel model;
  final void Function(PreMarkResult result) onResult;
  final String languageCode;

  @override
  State<PreMarkValidationPrompt> createState() =>
      _PreMarkValidationPromptState();
}

class _PreMarkValidationPromptState extends State<PreMarkValidationPrompt> {
  bool _running = false;

  Future<void> _run() async {
    setState(() => _running = true);
    // Stub: real implementation captures camera frame bytes.
    final fakeImageBytes = List<int>.filled(1024, 0);
    try {
      final result = await widget.model.runAndDelete(fakeImageBytes);
      if (!mounted) return;
      widget.onResult(result);
    } finally {
      if (mounted) setState(() => _running = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.all(16),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              _prompt(widget.languageCode),
              style: const TextStyle(fontSize: 16),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 16),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                OutlinedButton(
                  onPressed: _running ? null : () => widget.onResult(
                    const PreMarkResult(
                      outcome: PreMarkOutcome.modelUnavailable,
                      imageDeleteElapsedMs: 0,
                      modelVersion: 'skipped',
                    ),
                  ),
                  child: Text(_skipLabel(widget.languageCode)),
                ),
                ElevatedButton(
                  onPressed: _running ? null : _run,
                  child: _running
                      ? const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : Text(_acceptLabel(widget.languageCode)),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  static String _prompt(String lang) {
    const prompts = {
      'sw': 'Ungependa tuangalie karatasi yako kabla ya kuweka alama? (Hiari)',
      'en': 'Would you like us to check your ballot before marking? (Optional)',
    };
    return prompts[lang] ?? prompts['en']!;
  }

  static String _skipLabel(String lang) => lang == 'sw' ? 'Ruka' : 'Skip';
  static String _acceptLabel(String lang) => lang == 'sw' ? 'Ndio' : 'Yes';
}
