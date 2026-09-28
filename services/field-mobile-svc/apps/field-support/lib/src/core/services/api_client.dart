/// EelHttpClient factory for field-support.
library;

import 'package:dio/dio.dart';
import 'package:uuid/uuid.dart';

class EelHttpClientFactory {
  static EelHttpClient create({
    required String baseUrl, required String pinSetId,
    int connectTimeoutMs = 10000, int receiveTimeoutMs = 30000, int maxRetries = 3,
  }) {
    final dio = Dio(BaseOptions(
      baseUrl: baseUrl,
      connectTimeout: Duration(milliseconds: connectTimeoutMs),
      receiveTimeout: Duration(milliseconds: receiveTimeoutMs),
      followRedirects: false,
    ));
    dio.httpClientAdapter = EelHttpClientAdapter(pinSetId: pinSetId);
    dio.interceptors.add(RetryInterceptor(maxRetries: maxRetries, retryDelays: const [1, 2, 4, 8]));
    dio.interceptors.add(IdempotencyInterceptor());
    return EelHttpClient(innerDio: dio);
  }
}

class RetryInterceptor extends Interceptor {
  final int maxRetries;
  final List<int> retryDelays;
  const RetryInterceptor({required this.maxRetries, required this.retryDelays});
  @override void onError(DioException err, ErrorInterceptorHandler handler) async {
    final attempt = err.requestOptions.extra['retry_attempt'] as int? ?? 0;
    if (attempt >= maxRetries) return handler.next(err);
    final baseDelay = retryDelays[attempt.min(retryDelays.length - 1)];
    final jitter = DateTime.now().millisecondsSinceEpoch % 1000;
    await Future.delayed(Duration(milliseconds: (baseDelay * 1000) + jitter));
    err.requestOptions.extra['retry_attempt'] = attempt + 1;
    try {
      final r = await Dio().fetch(err.requestOptions); handler.resolve(r);
    } catch (_) { handler.next(err); }
  }
}

class IdempotencyInterceptor extends Interceptor {
  @override void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    if (!options.headers.containsKey('Idempotency-Key')) {
      options.headers['Idempotency-Key'] = const Uuid().v7();
    }
    handler.next(options);
  }
}

class EelHttpClient {
  final Dio innerDio;
  EelHttpClient({required this.innerDio});
  Future<Response> get(String p, {Map<String, dynamic>? q, Options? o}) => innerDio.get(p, queryParameters: q, options: o);
  Future<Response> post(String p, {dynamic d, Map<String, dynamic>? q, Options? o}) => innerDio.post(p, data: d, queryParameters: q, options: o);
  Future<Response> patch(String p, {dynamic d, Map<String, dynamic>? q, Options? o}) => innerDio.patch(p, data: d, queryParameters: q, options: o);
  Future<Response> delete(String p, {dynamic d, Map<String, dynamic>? q, Options? o}) => innerDio.delete(p, data: d, queryParameters: q, options: o);
}
