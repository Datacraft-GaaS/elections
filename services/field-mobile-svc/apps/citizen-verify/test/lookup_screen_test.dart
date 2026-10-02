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
import 'dart:typed_data';

import 'package:citizen_verify/src/core/services/otp_token.dart';
import 'package:citizen_verify/src/core/services/register_service.dart';
import 'package:citizen_verify/src/features/register/lookup_screen.dart';
import 'package:dio/dio.dart';
import 'package:eel_design/eel_design.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class _StubAdapter implements HttpClientAdapter {
  _StubAdapter({this.status = 200, required this.body});

  final int status;
  final Object? body;
  final sent = <String>[];
  String? authHeader;

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<Uint8List>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    sent.add('${options.method} ${options.path}');
    authHeader = options.headers['Authorization'] as String?;
    return ResponseBody.fromString(
      jsonEncode(body),
      status,
      headers: {
        Headers.contentTypeHeader: [Headers.jsonContentType],
      },
    );
  }

  @override
  void close({bool force = false}) {}
}

void main() {
  late _StubAdapter adapter;

  Future<void> pump(
    WidgetTester tester, {
    String? token = 'otp-token-1',
    int status = 200,
    Object? body = const <String, Object>{},
  }) async {
    adapter = _StubAdapter(status: status, body: body);
    final dio = Dio()..httpClientAdapter = adapter;
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          otpTokenProvider.overrideWith((ref) => token),
          registerServiceProvider.overrideWithValue(RegisterService(dio: dio)),
        ],
        child: const MaterialApp(home: RegisterLookupScreen()),
      ),
    );
    await tester.pumpAndSettle();
  }

  Future<void> fill(WidgetTester tester) async {
    await tester.enterText(find.byType(EelTextField).first, 'hash-abc');
    await tester.enterText(find.byType(EelTextField).last, 'My own record');
  }

  // The Verify Registration button had an empty handler, so the screen could
  // not be used at all. These pin that it calls the real service with the
  // OTP-issued bearer token.
  testWidgets('looks up with the OTP-issued bearer token', (tester) async {
    await pump(tester, body: {
      'found': true,
      'id_hash': 'hash-abc',
      'status': 'REGISTERED',
      'station_name': 'Nairobi Central',
      'ward_name': 'Kasarani',
    });
    await fill(tester);
    await tester.tap(find.widgetWithText(EelButton, 'Verify Registration'));
    await tester.pumpAndSettle();

    expect(adapter.sent, hasLength(1));
    expect(adapter.authHeader, 'Bearer otp-token-1');
    final card = find.byType(EelResultCard);
    expect(card, findsOneWidget);
    expect(
      find.descendant(
        of: card,
        matching: find.textContaining('REGISTERED'),
      ),
      findsOneWidget,
    );
    expect(
      find.descendant(
        of: card,
        matching: find.textContaining('Nairobi Central'),
      ),
      findsOneWidget,
    );
  });

  testWidgets('refuses without an OTP token and issues no request',
      (tester) async {
    await pump(tester, token: null);
    await fill(tester);
    await tester.tap(find.widgetWithText(EelButton, 'Verify Registration'));
    await tester.pumpAndSettle();

    expect(adapter.sent, isEmpty);
    expect(
      find.descendant(
        of: find.byType(EelBanner),
        matching: find.textContaining('Verify your phone number first'),
      ),
      findsOneWidget,
    );
  });

  testWidgets('requires the ID hash', (tester) async {
    await pump(tester);
    await tester.enterText(find.byType(EelTextField).last, 'My own record');
    await tester.tap(find.widgetWithText(EelButton, 'Verify Registration'));
    await tester.pumpAndSettle();

    expect(adapter.sent, isEmpty);
    expect(
      find.descendant(
        of: find.byType(EelBanner),
        matching: find.textContaining('hashed voter ID'),
      ),
      findsOneWidget,
    );
  });

  testWidgets('requires the written justification per M13 FR-010',
      (tester) async {
    await pump(tester);
    await tester.enterText(find.byType(EelTextField).first, 'hash-abc');
    await tester.tap(find.widgetWithText(EelButton, 'Verify Registration'));
    await tester.pumpAndSettle();

    expect(adapter.sent, isEmpty);
    expect(
      find.descendant(
        of: find.byType(EelBanner),
        matching: find.textContaining('written justification is required'),
      ),
      findsOneWidget,
    );
  });

  testWidgets('surfaces a rate limit rather than an empty card',
      (tester) async {
    await pump(tester, status: 429, body: <String, Object>{});
    await fill(tester);
    await tester.tap(find.widgetWithText(EelButton, 'Verify Registration'));
    await tester.pumpAndSettle();

    expect(find.byType(EelResultCard), findsNothing);
    expect(
      find.descendant(
        of: find.byType(EelBanner),
        matching: find.textContaining('Rate limited'),
      ),
      findsOneWidget,
    );
  });

  testWidgets('surfaces a miss without claiming a record', (tester) async {
    await pump(tester, status: 404, body: <String, Object>{});
    await fill(tester);
    await tester.tap(find.widgetWithText(EelButton, 'Verify Registration'));
    await tester.pumpAndSettle();

    expect(find.byType(EelResultCard), findsNothing);
    expect(
      find.descendant(
        of: find.byType(EelBanner),
        matching: find.textContaining('No record matched'),
      ),
      findsOneWidget,
    );
  });
}
