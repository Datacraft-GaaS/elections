import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class IncidentScreen extends ConsumerWidget {
	const IncidentScreen({super.key});

	@override
	Widget build(BuildContext context, WidgetRef ref) {
		return Scaffold(
			appBar: AppBar(title: const Text('Report Incident')),
			body: const Center(child: Text('Incident submission stub')),
		);
	}
}
