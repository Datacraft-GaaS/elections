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

/// EelHttpClient factory for party-agent.
library;

import 'package:dio/dio.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart' as kit;
import 'package:uuid/uuid.dart';

class EelHttpClientFactory {
  /// [pinSet] is passed rather than a pinSetId: resolving an id to pins is the
  /// caller's job, and a factory that looked one up would have to invent a
  /// trust store. A null pinSet means no pinning, so callers should prefer to
  /// supply one.
  static EelHttpClient create({
    required String baseUrl,
    required kit.EelAttestationClient attestation,
    kit.PinSet? pinSet,
    int connectTimeoutMs = 10000,
    int receiveTimeoutMs = 30000,
  }) {
    final client = kit.EelHttpClient(
      baseUrl: baseUrl,
      attestation: attestation,
      pinSet: pinSet,
      connectTimeout: Duration(milliseconds: connectTimeoutMs),
      receiveTimeout: Duration(milliseconds: receiveTimeoutMs),
    );
    // Retry and attestation headers come from the SDK client itself. Only the
    // idempotency key is this app's own policy.
    client.raw.interceptors.add(IdempotencyInterceptor());
    return EelHttpClient(client);
  }
}

class IdempotencyInterceptor extends Interceptor {
  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    if (!options.headers.containsKey('Idempotency-Key')) {
      options.headers['Idempotency-Key'] = const Uuid().v7();
    }
    handler.next(options);
  }
}

/// Thin adapter over the SDK's pinned, attested client.
///
/// This used to be an independent EelHttpClient wrapping a bare Dio, shadowing
/// the SDK class of the same name. Anything holding one of those talked to the
/// network with no SPKI pinning, no attestation header and no retry, which is
/// exactly the set of properties the SDK client exists to provide.
///
/// Every verb goes through the SDK client's Dio rather than its get/post
/// helpers, because those two helpers have narrower signatures than the rest of
/// Dio's API and the security interceptors (pinning adapter, attestation
/// header, retry) are all installed on that Dio either way.
class EelHttpClient {
  EelHttpClient(this._client);

  final kit.EelHttpClient _client;

  /// The pinned, attested kit-sdk client underneath.
  ///
  /// Services that take a kit.EelHttpClient directly take this, so they share
  /// the pinning and attestation the rest of the app uses rather than opening
  /// an unpinned connection of their own.
  kit.EelHttpClient get client => _client;

  Dio get _dio => _client.raw;

  Future<Response<T>> get<T>(String p, {Map<String, dynamic>? q, Options? o}) =>
      _dio.get<T>(p, queryParameters: q, options: o);

  Future<Response<T>> post<T>(String p, {dynamic data, Map<String, dynamic>? q, Options? o}) =>
      _dio.post<T>(p, data: data, queryParameters: q, options: o);

  Future<Response> patch(String p, {dynamic data, Map<String, dynamic>? q, Options? o}) =>
      _dio.patch(p, data: data, queryParameters: q, options: o);

  Future<Response> delete(String p, {dynamic data, Map<String, dynamic>? q, Options? o}) =>
      _dio.delete(p, data: data, queryParameters: q, options: o);
}
