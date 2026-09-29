import 'dart:convert';
import 'dart:typed_data';

import 'package:citizen_verify/src/core/services/results_service.dart';
import 'package:citizen_verify/src/features/results/results_screen.dart';
import 'package:dio/dio.dart';
import 'package:eel_design/eel_design.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

/// A Dio whose adapter answers with a canned body, so the screen can be driven
/// without a network. Records the requests it was asked for.
class _StubAdapter implements HttpClientAdapter {
  _StubAdapter({this.status = 200, required this.body});

  final int status;
  final Object? body;
  final sent = <String>[];

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<Uint8List>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    sent.add('${options.method} ${options.path}');
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

  Future<void> pump(WidgetTester tester, _StubAdapter stub) async {
    adapter = stub;
    final dio = Dio()..httpClientAdapter = stub;
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          resultsServiceProvider.overrideWithValue(ResultsService(dio: dio)),
        ],
        child: const MaterialApp(home: ResultsScreen()),
      ),
    );
    await tester.pumpAndSettle();
  }

  // The screen used to render a hardcoded "Station KE-NBI-001 … 87% • Verified"
  // card behind a dead Load Results button, so a citizen saw invented results
  // presented as verified. These pin that it shows nothing until it fetches.
  testWidgets('shows no results before a fetch', (tester) async {
    await pump(tester, _StubAdapter(body: <String, Object>{}));
    expect(find.byType(EelResultCard), findsNothing);
    expect(find.textContaining('87%'), findsNothing);
    expect(find.text('Station KE-NBI-001'), findsNothing);
  });

  testWidgets('renders what the service returns', (tester) async {
    await pump(tester, _StubAdapter(body: {
      'station_code': 'KE-NBI-077',
      'total_voters': 2000,
      'total_turnout': 1250,
      'proof': {'is_valid': true, 'root_hash': 'abc123'},
    }));

    await tester.enterText(find.byType(EelTextField), 'KE-NBI-077');
    await tester.tap(find.widgetWithText(EelButton, 'Load Results'));
    await tester.pumpAndSettle();

    // Scoped to the card: the station code also sits in the text field, so an
    // unscoped find matches twice.
    final card = find.byType(EelResultCard);
    expect(card, findsOneWidget);
    expect(
      find.descendant(of: card, matching: find.text('KE-NBI-077')),
      findsOneWidget,
    );
    expect(
      find.descendant(
        of: card,
        matching: find.textContaining('1250 / Expected: 2000'),
      ),
      findsOneWidget,
    );
    expect(
      find.descendant(
        of: card,
        matching: find.textContaining('Proof verified on device'),
      ),
      findsOneWidget,
    );
    expect(adapter.sent, hasLength(1));
  });

  testWidgets('says so when the proof will not verify', (tester) async {
    await pump(tester, _StubAdapter(body: {
      'station_code': 'KE-NBI-078',
      'proof': {'is_valid': false, 'root_hash': 'abc123'},
    }));

    await tester.enterText(find.byType(EelTextField), 'KE-NBI-078');
    await tester.tap(find.widgetWithText(EelButton, 'Load Results'));
    await tester.pumpAndSettle();

    expect(
      find.descendant(
        of: find.byType(EelResultCard),
        matching: find.textContaining('Proof could not be verified'),
      ),
      findsOneWidget,
    );
  });

  testWidgets('surfaces a failed fetch instead of stale figures',
      (tester) async {
    await pump(
      tester,
      _StubAdapter(status: 500, body: <String, Object>{'error': 'nope'}),
    );

    await tester.enterText(find.byType(EelTextField), 'KE-NBI-079');
    await tester.tap(find.widgetWithText(EelButton, 'Load Results'));
    await tester.pumpAndSettle();

    expect(find.byType(EelResultCard), findsNothing);
    expect(find.textContaining('500'), findsOneWidget);
  });

  testWidgets('refuses an empty station code without calling the API',
      (tester) async {
    await pump(tester, _StubAdapter(body: <String, Object>{}));

    await tester.tap(find.widgetWithText(EelButton, 'Load Results'));
    await tester.pumpAndSettle();

    expect(find.text('Enter a station code.'), findsOneWidget);
    expect(adapter.sent, isEmpty);
  });
}
