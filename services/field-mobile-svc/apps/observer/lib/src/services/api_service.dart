import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class ApiService {
	final Dio _dio;

	ApiService({String baseUrl = 'https://bff.eel.internal'})
		: _dio = Dio(BaseOptions(
				baseUrl: baseUrl,
				connectTimeout: const Duration(seconds: 10),
				receiveTimeout: const Duration(seconds: 30),
			));

	Dio get client => _dio;
}

final apiServiceProvider = Provider<ApiService>((ref) => ApiService());
