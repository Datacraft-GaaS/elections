import 'package:flutter_test/flutter_test.dart';
import 'package:eel_kit_sdk/main.dart';

void main() {
  testWidgets('example app renders the SDK demo screen', (tester) async {
    await tester.pumpWidget(const EelExampleApp());
    expect(find.byType(EelExampleApp), findsOneWidget);
    expect(find.text('Attest Device'), findsOneWidget);
  });
}
