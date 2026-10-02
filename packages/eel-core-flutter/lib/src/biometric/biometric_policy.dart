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

/// BiometricCapturePolicy — settable thresholds from policy bundle.
/// Handles environmental degradation (sunlight), individual edge cases (missing
/// eye/finger), and ethnicity-correlated iris contrast variation.
/// All values come from the BFF policy bundle (bootstrap), not hard-coded.
class BiometricCapturePolicy {
	/// Minimum acceptable quality score for fingerprint (0–100).
	/// Default 60; bright-sunlight field conditions may require 50.
	final int fpMinQuality;

	/// Minimum acceptable quality score for iris (0–100).
	/// Ginger/redhead irises have naturally lower contrast (melanin).
	/// Default 55; field conditions may require 45.
	final int irisMinQuality;

	/// Retry limit per biometric capture attempt.
	/// Each retry re-lights/re-positions; operator guided.
	final int maxRetries;

	/// Capture timeout per attempt (ms).
	final int captureTimeoutMs;

	/// Fallback finger positions when standard positions unavailable.
	/// Indexed by standard position → list of fallbacks.
	final Map<String, List<String>> fpFallbackPositions;

	/// Whether single-eye mode is allowed (one eye missing/damaged).
	final bool allowSingleEyeMode;

	/// Whether to allow use of a previously enrolled template as fallback
	/// when all available positions fail quality threshold.
	/// Requires prior enrollment record.
	final bool allowPriorTemplateFallback;

	/// Outdoor/indoor hint for threshold adjustment.
	/// "outdoor" → lower thresholds (more tolerant of degradation).
	/// Set by operator at session start; not auto-detected.
	final String environmentHint;

	const BiometricCapturePolicy({
		this.fpMinQuality = 60,
		this.irisMinQuality = 55,
		this.maxRetries = 3,
		this.captureTimeoutMs = 5000,
		this.fpFallbackPositions = const {
			'LEFT_INDEX': ['LEFT_MIDDLE', 'LEFT_RING'],
			'RIGHT_INDEX': ['RIGHT_MIDDLE', 'RIGHT_RING'],
			'LEFT_THUMB': ['LEFT_INDEX', 'RIGHT_THUMB'],
			'RIGHT_THUMB': ['RIGHT_INDEX', 'LEFT_THUMB'],
		},
		this.allowSingleEyeMode = true,
		this.allowPriorTemplateFallback = false,
		this.environmentHint = 'indoor',
	});

	/// Adjust iris threshold downward for field conditions.
	/// Called when operator selects "outdoor/bright light" mode.
	BiometricCapturePolicy withEnvironmentHint(String hint) => BiometricCapturePolicy(
		fpMinQuality: hint == 'outdoor' ? (fpMinQuality - 10).clamp(30, 100) : fpMinQuality,
		irisMinQuality: hint == 'outdoor' ? (irisMinQuality - 15).clamp(25, 100) : irisMinQuality,
		maxRetries: maxRetries,
		captureTimeoutMs: captureTimeoutMs,
		fpFallbackPositions: fpFallbackPositions,
		allowSingleEyeMode: allowSingleEyeMode,
		allowPriorTemplateFallback: allowPriorTemplateFallback,
		environmentHint: hint,
	);

	/// Adjust iris threshold for lower-contrast iris.
	/// Approximated by hair color proxy (self-reported by operator, never forced).
	/// Ginger/redhead → lower threshold. Policy overrides this.
	BiometricCapturePolicy withIrisContrastProxy(double contrastProxy) =>
		BiometricCapturePolicy(
			fpMinQuality: fpMinQuality,
			irisMinQuality: (irisMinQuality - (1 - contrastProxy).round() * 15).clamp(25, 100),
			maxRetries: maxRetries,
			captureTimeoutMs: captureTimeoutMs,
			fpFallbackPositions: fpFallbackPositions,
			allowSingleEyeMode: allowSingleEyeMode,
			allowPriorTemplateFallback: allowPriorTemplateFallback,
			environmentHint: environmentHint,
		);

	/// Apply missing-position override from operator declaration.
	/// E.g., operator marks "RIGHT_INDEX: missing" → skip to fallback.
	BiometricCapturePolicy withMissingPositions(Map<String, bool> missing) {
		final fallbackMap = Map<String, List<String>>.from(fpFallbackPositions);
		for (final entry in missing.entries) {
			if (entry.value == true && fallbackMap.containsKey(entry.key)) {
				// Promote first fallback to primary
				final fallbacks = fallbackMap[entry.key]!;
				fallbackMap[entry.key] = [fallbacks.first];
			}
		}
		return BiometricCapturePolicy(
			fpMinQuality: fpMinQuality,
			irisMinQuality: irisMinQuality,
			maxRetries: maxRetries,
			captureTimeoutMs: captureTimeoutMs,
			fpFallbackPositions: fallbackMap,
			allowSingleEyeMode: allowSingleEyeMode,
			allowPriorTemplateFallback: allowPriorTemplateFallback,
			environmentHint: environmentHint,
		);
	}

	/// Load from policy bundle JSON (BFF bootstrap response).
	factory BiometricCapturePolicy.fromBundle(Map<String, dynamic> bundle) {
		final fp = bundle['biometric'] as Map<String, dynamic>? ?? {};
		return BiometricCapturePolicy(
			fpMinQuality: (fp['fpMinQuality'] as num?)?.toInt() ?? 60,
			irisMinQuality: (fp['irisMinQuality'] as num?)?.toInt() ?? 55,
			maxRetries: (fp['maxRetries'] as num?)?.toInt() ?? 3,
			captureTimeoutMs: (fp['captureTimeoutMs'] as num?)?.toInt() ?? 5000,
			fpFallbackPositions: (fp['fpFallbackPositions'] as Map?)?.map(
				(k, v) => MapEntry(k, (v as List).cast<String>()),
			) ?? const {},
			allowSingleEyeMode: fp['allowSingleEyeMode'] as bool? ?? true,
			allowPriorTemplateFallback: fp['allowPriorTemplateFallback'] as bool? ?? false,
			environmentHint: 'indoor',
		);
	}

	/// Operator-selectable presets for field conditions.
	static const outdoorPreset = BiometricCapturePolicy(
		fpMinQuality: 50,
		irisMinQuality: 40,
		maxRetries: 4,
		captureTimeoutMs: 7000,
		environmentHint: 'outdoor',
	);
}

/// Result of a single biometric capture attempt (fingerprint OR iris).
class BiometricCaptureResult {
	final bool isValid;
	final int qualityScore;         // 0–100 from FAP60 / AntIris
	final bool meetsThreshold;      // >= policy threshold
	final String? templateBase64;  // ISO 19794-2 (fp) or ISO 19794-6 (iris)
	final String? position;         // "LEFT_INDEX", "RIGHT_EYE", etc.
	final String? error;
	final int retriesUsed;

	const BiometricCaptureResult({
		required this.isValid,
		required this.qualityScore,
		required this.meetsThreshold,
		this.templateBase64,
		this.position,
		this.error,
		this.retriesUsed = 0,
	});

	factory BiometricCaptureResult.invalid(String error) => BiometricCaptureResult(
		isValid: false, qualityScore: 0, meetsThreshold: false, error: error,
	);
}
