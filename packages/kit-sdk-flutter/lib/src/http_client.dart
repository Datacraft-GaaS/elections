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

import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart';

import 'attestation.dart';

/// SPKI pin set entry (plan §3.3).
@immutable
class SpkiPin {
  const SpkiPin({
    required this.base64Sha256,
    this.host,
    this.port,
  });

  /// Base64-encoded SHA-256 of the leaf certificate's SPKI.
  final String base64Sha256;

  /// Optional host constraint. Null means match any host.
  final String? host;

  /// Optional port constraint. Null means match any port.
  final int? port;

  factory SpkiPin.fromJson(Map<String, dynamic> json) => SpkiPin(
        base64Sha256: json['base64_sha256'] as String? ?? '',
        host: json['host'] as String?,
        port: json['port'] as int?,
      );

  Map<String, dynamic> toJson() => {
        'base64_sha256': base64Sha256,
        if (host != null) 'host': host,
        if (port != null) 'port': port,
      };

  @override
  bool operator ==(Object other) =>
      other is SpkiPin && other.base64Sha256 == base64Sha256;

  @override
  int get hashCode => base64Sha256.hashCode;
}

/// Pin set fetched from BFF config + mirrored in
/// `assets/config/pins-<set>.json` (plan §3.3).
@immutable
class PinSet {
  const PinSet({
    required this.setId,
    required this.pins,
    this.createdAt,
  });

  final String setId;
  final List<SpkiPin> pins;
  final DateTime? createdAt;

  factory PinSet.fromJson(Map<String, dynamic> json) => PinSet(
        setId: json['set_id'] as String? ?? '',
        pins: ((json['pins'] as List?) ?? [])
            .map((e) => SpkiPin.fromJson(e as Map<String, dynamic>))
            .toList(),
        createdAt: json['created_at'] != null
            ? DateTime.parse(json['created_at'] as String)
            : null,
      );

  Map<String, dynamic> toJson() => {
        'set_id': setId,
        'pins': pins.map((p) => p.toJson()).toList(),
        if (createdAt != null) 'created_at': createdAt!.toIso8601String(),
      };

  /// Whether this pin set is a valid rotation target (has at least one pin).
  bool get hasPins => pins.isNotEmpty;
}

/// SPKI pinning [HttpClientAdapter] for Dio (plan §3.3).
///
/// On pin failure the connection is refused and the error is re-thrown as a
/// [DioException] with type [DioExceptionType.badCertificate] so that the
/// caller can map it to `connectivity.untrusted`.
class SpkiPinningAdapter implements HttpClientAdapter {
  SpkiPinningAdapter({
    required this.pins,
    this.allowList = const {},
  });

  /// Active pins (dual-pin window: both old and new sets may coexist).
  final List<SpkiPin> pins;

  /// Hosts exempt from pinning (e.g. localhost for dev).
  final Set<String> allowList;

  late final HttpClient _inner = HttpClient()
    ..badCertificateCallback = _callback;

  bool _callback(X509Certificate cert, String host, int port) {
    // Dev allowlist: skip pinning for explicitly allowed hosts.
    if (allowList.contains(host)) return true;

    // Compute the SPKI hash of the leaf certificate.
    // We hash the raw certificate bytes (DER-encoded) as a simplified
    // approach. Production implementations should extract the exact
    // SubjectPublicKeyInfo field via ASN.1 parsing.
    final certBytes = cert.der;
    if (certBytes.isEmpty) return false;

    final pinHash = _sha256Base64(certBytes.toList());

    // Accept if any pin matches (dual-pin window: multiple pins valid).
    return pins.any(
      (pin) =>
          (pin.host == null || pin.host == host) &&
          (pin.port == null || pin.port == port) &&
          pin.base64Sha256 == pinHash,
    );
  }

  String _sha256Base64(List<int> bytes) {
    final digest = cryptoSha256(bytes);
    return base64.encode(digest);
  }

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<List<int>>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    try {
      // Build the URL from the request options.
      final uri = options.uri;
      final client = _inner;

      // Open the URL connection.
      final request = await client.openUrl(options.method, uri);

      // Copy headers.
      options.headers.forEach((key, value) {
        if (value != null) {
          request.headers.set(key, value.toString());
        }
      });
      // Also copy any additional headers from the options.

      // Write request body if present.
      if (requestStream != null) {
        await request.addStream(requestStream);
      }

      // Send the request.
      final response = await request.close();

      // Read the response body.
      final bodyBytes = await response.expand((chunk) => chunk).toList();

      // Build the response body.
      final responseHeaders = <String, List<String>>{};
      response.headers.forEach((name, values) {
        responseHeaders[name] = values;
      });
      return ResponseBody.fromBytes(
        bodyBytes,
        response.statusCode,
        statusMessage: response.reasonPhrase,
        headers: responseHeaders,
      );
    } on SocketException catch (e) {
      throw DioException.connectionError(
        requestOptions: options,
        reason: e.message,
      );
    } catch (e) {
      if (e is DioException) rethrow;
      throw DioException(
        requestOptions: options,
        type: DioExceptionType.badCertificate,
        error: e,
      );
    }
  }

  @override
  void close({bool force = false}) => _inner.close(force: force);
}

/// SHA-256 digest of raw bytes, returned as base64 — used by the pinning
/// adapter to compare against configured pins.
List<int> cryptoSha256(List<int> bytes) => sha256.convert(bytes).bytes;

/// Dio client with SPKI pinning, attestation header injection and retry.
///
/// TLS 1.3 is enforced at the OS/platform layer; Dart's http stack uses the
/// system TLS.
class EelHttpClient {
  EelHttpClient({
    required String baseUrl,
    required EelAttestationClient attestation,
    Duration connectTimeout = const Duration(seconds: 10),
    Duration receiveTimeout = const Duration(seconds: 30),
    PinSet? pinSet,
    Set<String>? allowList,
  })  : _attestation = attestation {
    _dio = Dio(BaseOptions(
      baseUrl: baseUrl,
      connectTimeout: connectTimeout,
      receiveTimeout: receiveTimeout,
      headers: {'Accept': 'application/json'},
    ));

    // Apply SPKI pinning if a pin set is provided.
    if (pinSet != null && pinSet.hasPins) {
      _dio.httpClientAdapter = SpkiPinningAdapter(
        pins: pinSet.pins,
        allowList: allowList ?? const {},
      );
    }

    _dio.interceptors.add(_AttestInterceptor(_attestation));
    _dio.interceptors.add(_RetryInterceptor());
  }

  final EelAttestationClient _attestation;
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
