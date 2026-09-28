/// EelHttpClient factory for party-agent.

import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:dio/dio.dart';
import 'package:uuid/uuid.dart';

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
      followRedirects: false,
    ));

    dio.httpClientAdapter = EelHttpClientAdapter(pinSetId: pinSetId);
    dio.interceptors.add(RetryInterceptor(
      maxRetries: maxRetries,
      retryDelays: const [1, 2, 4, 8],
      retryCondition: (error, attempt) =>
        error is DioException &&
        (error.type == DioExceptionType.connectionTimeout ||
         error.type == DioExceptionType.receiveTimeout ||
         error.type == DioExceptionType.connectionError),
    ));
    dio.interceptors.add(IdempotencyInterceptor());

    return EelHttpClient(innerDio: dio);
  }
}

class RetryInterceptor extends Interceptor {
  final int maxRetries;
  final List<int> retryDelays;

  const RetryInterceptor({required this.maxRetries, required this.retryDelays});

  @override
  void onError(DioException err, ErrorInterceptorHandler handler) async {
    final attempt = err.requestOptions.extra['retry_attempt'] as int? ?? 0;
    if (attempt >= maxRetries) return handler.next(err);

    final baseDelay = retryDelays[attempt.min(retryDelays.length - 1)];
    final jitter = DateTime.now().millisecondsSinceEpoch % 1000;
    final delayMs = (baseDelay * 1000) + jitter;
    await Future.delayed(Duration(milliseconds: delayMs));

    err.requestOptions.extra['retry_attempt'] = attempt + 1;
    try {
      final response = await Dio().fetch(err.requestOptions);
      handler.resolve(response);
    } catch (e) {
      handler.next(err);
    }
  }
}

class IdempotencyInterceptor extends Interceptor {
  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    if (options.headers.containsKey('Idempotency-Key')) {
      return handler.next(options);
    }
    options.headers['Idempotency-Key'] = const Uuid().v7();
    handler.next(options);
  }
}

class EelHttpClient {
  final Dio innerDio;
  EelHttpClient({required this.innerDio});

  Future<Response> get(String path, {Map<String, dynamic>? queryParameters, Options? options}) =>
    innerDio.get(path, queryParameters: queryParameters, options: options);
  Future<Response> post(String path, {dynamic data, Map<String, dynamic>? queryParameters, Options? options}) =>
    innerDio.post(path, data: data, queryParameters: queryParameters, options: options);
  Future<Response> patch(String path, {dynamic data, Map<String, dynamic>? queryParameters, Options? options}) =>
    innerDio.patch(path, data: data, queryParameters: queryParameters, options: options);
  Future<Response> delete(String path, {dynamic data, Map<String, dynamic>? queryParameters, Options? options}) =>
    innerDio.delete(path, data: data, queryParameters: queryParameters, options: options);
}
