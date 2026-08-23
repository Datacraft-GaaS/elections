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
