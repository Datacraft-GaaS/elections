import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

Widget _harness({required GlobalKey<NavigatorState> key}) => MaterialApp(
      navigatorKey: key,
      home: const Scaffold(body: Text('home')),
    );

void main() {
  setUp(() {
    EelErrorLog.clearForTest();
    EelErrorOverlay.navigatorOverride = null;
    EelErrorOverlay.resetForTest();
  });

  tearDown(() {
    EelErrorOverlay.navigatorOverride = null;
  });

  test('support IDs are stable, formatted, and carry no payload', () {
    final at = DateTime.utc(2026, 9, 18, 12);
    final a = eelSupportId(StateError('x'), at: at);
    final b = eelSupportId(StateError('x'), at: at);
    expect(a, b);
    expect(a, matches(RegExp(r'^EEL-[0-9A-Z]+-[0-9A-F]{4}$')));
    expect(a, isNot(contains('x')));
    expect(
      eelSupportId(StateError('y'), at: at),
      isNot(eelSupportId(ArgumentError('y'), at: at)),
    );
  });

  testWidgets('zone error records + banners without breaking the app',
      (tester) async {
    final key = GlobalKey<NavigatorState>();
    EelErrorOverlay.navigatorOverride = key;
    final reported = <EelErrorRecord>[];
    await tester.pumpWidget(_harness(key: key));
    await tester.pumpAndSettle();

    handleZoneError(StateError('boom'), StackTrace.empty,
        onError: reported.add);
    await tester.pump();

    expect(find.text('Something went wrong'), findsOneWidget);
    expect(find.textContaining('Error EEL-'), findsOneWidget);
    expect(find.text('home'), findsOneWidget);
    expect(EelErrorLog.all, hasLength(1));
    expect(EelErrorLog.all.single.fatal, isFalse);
    expect(reported, hasLength(1));
    expect(reported.single.supportId,
        contains(EelErrorLog.all.single.supportId));
    await tester.pump(const Duration(seconds: 9));
  });

  testWidgets('banner tap opens the fatal screen with support ID',
      (tester) async {
    final key = GlobalKey<NavigatorState>();
    EelErrorOverlay.navigatorOverride = key;
    await tester.pumpWidget(_harness(key: key));
    await tester.pumpAndSettle();

    final record =
        handleZoneError(StateError('boom'), StackTrace.empty);
    await tester.pump();
    await tester.tap(find.textContaining('Error EEL-'));
    await tester.pumpAndSettle();

    expect(find.byType(EelFatalErrorScreen), findsOneWidget);
    expect(find.textContaining(record.supportId), findsOneWidget);
    expect(find.text('Restart app'), findsOneWidget);
    await tester.pump(const Duration(seconds: 9));
  });

  testWidgets('framework error replaces the route stack', (tester) async {
    final key = GlobalKey<NavigatorState>();
    EelErrorOverlay.navigatorOverride = key;
    await tester.pumpWidget(_harness(key: key));
    await tester.pumpAndSettle();

    handleFlutterError(
      FlutterErrorDetails(exception: StateError('render boom')),
    );
    await tester.pumpAndSettle();

    expect(find.byType(EelFatalErrorScreen), findsOneWidget);
    expect(find.text('home'), findsNothing);
    expect(EelErrorLog.all.single.fatal, isTrue);
  });

  testWidgets('re-entrant fatal calls show a single screen', (tester) async {
    final key = GlobalKey<NavigatorState>();
    EelErrorOverlay.navigatorOverride = key;
    await tester.pumpWidget(_harness(key: key));
    await tester.pumpAndSettle();

    handleFlutterError(FlutterErrorDetails(exception: StateError('one')));
    handleFlutterError(FlutterErrorDetails(exception: StateError('two')));
    await tester.pumpAndSettle();

    expect(find.byType(EelFatalErrorScreen), findsOneWidget);
    // Both recorded, only one displayed.
    expect(EelErrorLog.all, hasLength(2));
    await tester.pump(const Duration(seconds: 9));
  });

  testWidgets('fatal screen details toggle reveals the error type',
      (tester) async {
    final key = GlobalKey<NavigatorState>();
    EelErrorOverlay.navigatorOverride = key;
    await tester.pumpWidget(_harness(key: key));
    await tester.pumpAndSettle();

    handleFlutterError(
      FlutterErrorDetails(exception: StateError('render boom')),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Details'));
    await tester.pump();

    expect(find.textContaining('StateError'), findsOneWidget);
  });
}
