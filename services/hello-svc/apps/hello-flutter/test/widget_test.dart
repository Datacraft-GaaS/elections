import 'package:flutter_test/flutter_test.dart';
import 'package:hello_flutter/main.dart';

void main() {
  testWidgets('renders ok: true text', (WidgetTester tester) async {
    await tester.pumpWidget(const EelHelloApp());
    expect(find.text('ok: true'), findsOneWidget);
  });
}
