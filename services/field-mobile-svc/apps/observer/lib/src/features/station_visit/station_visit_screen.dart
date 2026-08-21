import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class StationVisitScreen extends ConsumerWidget {
	const StationVisitScreen({super.key});

	@override
	Widget build(BuildContext context, WidgetRef ref) {
		return Scaffold(
			appBar: AppBar(title: const Text('Station Visit Log')),
			body: const Center(child: Text('GPS station visit log stub')),
		);
	}
}
