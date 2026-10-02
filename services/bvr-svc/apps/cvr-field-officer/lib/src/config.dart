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

/// Compile-time configuration resolved from --dart-define entries.
///
/// Field builds override the defaults at build time:
///   --dart-define=EEL_CVR_API_BASE=https://cvr.eel.internal
///   --dart-define=EEL_OCR_BASE=http://localhost:8091
library;

class AppConfig {
  const AppConfig._();

  static const String _apiBase = String.fromEnvironment(
    'EEL_CVR_API_BASE',
    defaultValue: 'https://cvr.eel.internal',
  );

  static const String _ocrBase = String.fromEnvironment(
    'EEL_OCR_BASE',
    defaultValue: 'http://localhost:8091',
  );

  /// Base URI of the bvr-svc CVR API.
  static Uri get apiBaseUri => Uri.parse(_apiBase);

  /// Base URI of the on-device OCR service (py/cvr-ocr sidecar).
  static Uri get ocrBaseUri => Uri.parse(_ocrBase);
}
