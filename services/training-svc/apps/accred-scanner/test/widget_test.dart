import 'package:flutter_test/flutter_test.dart';
import 'package:eel_accred_scanner/app.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

void main() {
  testWidgets('app renders without crashing', (tester) async {
    await tester.pumpWidget(
      const ProviderScope(child: EelAccredScannerApp()),
    );
    // ScanScreen requires camera, so expect at minimum no immediate crash
    await tester.pump();
  });
}
