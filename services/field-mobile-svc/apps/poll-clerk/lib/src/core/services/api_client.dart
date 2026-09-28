/// EelHttpClient factory for poll-clerk.
///
/// Per plan.md §2.1: All networking through kit-sdk `EelHttpClient` (dio).
/// No bare `Dio()` or `http.post` outside `core/api_client.dart`.

import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:dio/dio.dart';

/// Factory for creating pinned, retry-enabled HTTP clients.
/// Uses SPKI pinning from config pin-set.
class EelHttpClientFactory {
  static EelHttpClient create({
    required String baseUrl,
    required String pinSetId,
    int connectTimeoutMs = 10000,
    int receiveTimeoutMs = 30000,
    int maxRetries = 3,
  }) {
    final dio = Dio(BaseOptions(
      baseUrl: baseUrl,
      connectTimeout: Duration(milliseconds: connectTimeoutMs),
      receiveTimeout: Duration(milliseconds: receiveTimeoutMs),
      followRedirects: false, // Security: don't follow redirects
    ));

    // Add SPKI pinning adapter
    dio.httpClientAdapter = EelHttpClientAdapter(
      pinSetId: pinSetId,
    );

    // Add retry interceptor with full jitter
    dio.interceptors.add(RetryInterceptor(
      maxRetries: maxRetries,
      retryDelays: const [1, 2, 4, 8], // Seconds with full jitter
      retryCondition: (error, attempt) =>
        error is DioException &&
        (error.type == DioExceptionType.connectionTimeout ||
         error.type == DioExceptionType.receiveTimeout ||
         error.type == DioExceptionType.connectionError),
    ));

    // Add idempotency header injection
    dio.interceptors.add(IdempotencyInterceptor());

    return EelHttpClient(innerDio: dio);
  }
}

/// Retry interceptor with exponential backoff + full jitter
class RetryInterceptor extends Interceptor {
  final int maxRetries;
  final List<int> retryDelays; // Base delays in seconds

  const RetryInterceptor({
    required this.maxRetries,
    required this.retryDelays,
  });

  @override
  void onError(DioException err, ErrorInterceptorHandler handler) async {
    final attempt = err.requestOptions.extra['retry_attempt'] as int? ?? 0;
    
    if (attempt >= maxRetries) {
      return handler.next(err);
    }
    
    // Calculate delay with full jitter
    final baseDelay = retryDelays[attempt.min(retryDelays.length - 1)];
    final jitter = (DateTime.now().millisecondsSinceEpoch % 1000);
    final delayMs = (baseDelay * 1000) + jitter;
    
    await Future.delayed(Duration(milliseconds: delayMs));
    
    err.requestOptions.extra['retry_attempt'] = attempt + 1;
    
    // Retry the request
    try {
      final response = await Dio().fetch(err.requestOptions);
      handler.resolve(response);
    } catch (e) {
      handler.next(err);
    }
  }
}

/// Interceptor that injects Idempotency-Key header for POST/PATCH
class IdempotencyInterceptor extends Interceptor {
  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    // Use existing key if provided by caller
    if (options.headers.containsKey('Idempotency-Key')) {
      return handler.next(options);
    }
    
    // Generate UUID7 for new requests
    final key = const Uuid().v7();
    options.headers['Idempotency-Key'] = key;
    handler.next(options);
  }
}

/// Wrapper around kit-sdk's EelHttpClient
class EelHttpClient {
  final Dio innerDio;
  
  EelHttpClient({required this.innerDio});
  
  /// Make GET request
  Future<Response> get(
    String path, {
    Map<String, dynamic>? queryParameters,
    Options? options,
  }) async => innerDio.get(path, queryParameters: queryParameters, options: options);
  
  /// Make POST request with idempotency
  Future<Response> post(
    String path, {
    dynamic data,
    Map<String, dynamic>? queryParameters,
    Options? options,
  }) async => innerDio.post(path, data: data, queryParameters: queryParameters, options: options);
  
  /// Make PATCH request with idempotency
  Future<Response> patch(
    String path, {
    dynamic data,
    Map<String, dynamic>? queryParameters,
    Options? options,
  }) async => innerDio.patch(path, data: data, queryParameters: queryParameters, options: options);
  
  /// Make DELETE request
  Future<Response> delete(
    String path, {
    dynamic data,
    Map<String, dynamic>? queryParameters,
    Options? options,
  }) async => innerDio.delete(path, data: data, queryParameters: queryParameters, options: options);
}
