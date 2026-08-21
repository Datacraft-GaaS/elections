import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class StationDashboardScreen extends ConsumerWidget {
	const StationDashboardScreen({super.key});

	@override
	Widget build(BuildContext context, WidgetRef ref) {
		return Scaffold(
			appBar: AppBar(title: const Text('Station Dashboard')),
			body: const Center(child: Text('Assigned station mirror stub')),
		);
	}
}
