import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_party_agent/main.dart';

void main() {
	testWidgets('app renders without crash', (tester) async {
		await tester.pumpWidget(const ProviderScope(child: EelPartyAgentApp()));
		await tester.pump();
		expect(find.byType(MaterialApp), findsOneWidget);
	});
}
