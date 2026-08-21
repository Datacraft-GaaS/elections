import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class EodReportScreen extends ConsumerWidget {
	const EodReportScreen({super.key});

	@override
	Widget build(BuildContext context, WidgetRef ref) {
		return Scaffold(
			appBar: AppBar(title: const Text('End-of-Day Report')),
			body: const Center(child: Text('EOD signed report stub')),
		);
	}
}
