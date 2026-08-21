import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class MissionExportScreen extends ConsumerWidget {
	const MissionExportScreen({super.key});

	@override
	Widget build(BuildContext context, WidgetRef ref) {
		return Scaffold(
			appBar: AppBar(title: const Text('Export Mission PDF')),
			body: const Center(child: Text('Mission PDF export stub')),
		);
	}
}
