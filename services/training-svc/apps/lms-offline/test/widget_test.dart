import 'package:flutter_test/flutter_test.dart';
import 'package:eel_lms_offline/app.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

void main() {
  testWidgets('app renders without crashing', (tester) async {
    await tester.pumpWidget(
      const ProviderScope(child: EelLmsOfflineApp()),
    );
    expect(find.text('Training Modules'), findsOneWidget);
  });
}
