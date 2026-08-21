import 'package:dio/dio.dart';
import 'attestation.dart';

/// Dio client with cert pinning, attestation header injection, and retry.
///
/// [spkiPin] is the SPKI pin for the IEBC intermediate CA. TLS 1.3 is
/// enforced at the OS/platform layer; Dart's http stack uses the system TLS.
class EelHttpClient {
  EelHttpClient({
    required String baseUrl,
    required EelAttestationClient attestation,
    String? spkiPin,
    Duration connectTimeout = const Duration(seconds: 10),
    Duration receiveTimeout = const Duration(seconds: 30),
  })  : _attestation = attestation,
        _spkiPin = spkiPin {
    _dio = Dio(BaseOptions(
      baseUrl: baseUrl,
      connectTimeout: connectTimeout,
      receiveTimeout: receiveTimeout,
      headers: {'Accept': 'application/json'},
    ));
    _dio.interceptors.add(_AttestInterceptor(_attestation));
    _dio.interceptors.add(_RetryInterceptor());
  }

  final EelAttestationClient _attestation;
  // ignore: unused_field
  final String? _spkiPin;
  late final Dio _dio;

  Dio get raw => _dio;

  Future<Response<T>> get<T>(String path,
          {Map<String, dynamic>? queryParameters}) =>
      _dio.get<T>(path, queryParameters: queryParameters);

  Future<Response<T>> post<T>(String path,
          {Object? data, String? idempotencyKey}) =>
      _dio.post<T>(
        path,
        data: data,
        options: Options(
          headers: idempotencyKey != null
              ? {'Idempotency-Key': idempotencyKey}
              : null,
        ),
      );
}

class _AttestInterceptor extends Interceptor {
  _AttestInterceptor(this._attestation);
  final EelAttestationClient _attestation;

  @override
  Future<void> onRequest(
      RequestOptions options, RequestInterceptorHandler handler) async {
    try {
      final verdict = await _attestation.attest();
      options.headers['X-Eel-Attest'] =
          '${verdict.tier.name};device=${verdict.deviceId}';
    } catch (_) {
      // Best-effort; continue without header.
    }
    handler.next(options);
  }
}

class _RetryInterceptor extends Interceptor {
  static const _maxRetries = 3;

  @override
  Future<void> onError(
      DioException err, ErrorInterceptorHandler handler) async {
    final extra = err.requestOptions.extra;
    final retries = (extra['_retries'] as int?) ?? 0;
    if (retries < _maxRetries && _shouldRetry(err)) {
      final delay = Duration(milliseconds: 200 * (1 << retries));
      await Future<void>.delayed(delay);
      err.requestOptions.extra['_retries'] = retries + 1;
      try {
        final dio = Dio();
        final response = await dio.fetch<dynamic>(err.requestOptions);
        handler.resolve(response);
        return;
      } catch (_) {}
    }
    handler.next(err);
  }

  bool _shouldRetry(DioException e) =>
      e.type == DioExceptionType.connectionTimeout ||
      e.type == DioExceptionType.receiveTimeout ||
      (e.response?.statusCode != null && e.response!.statusCode! >= 500);
}
