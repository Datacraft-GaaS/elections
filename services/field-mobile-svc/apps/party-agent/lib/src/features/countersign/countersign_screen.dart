import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class CountersignScreen extends ConsumerWidget {
	const CountersignScreen({super.key});

	@override
	Widget build(BuildContext context, WidgetRef ref) {
		return Scaffold(
			appBar: AppBar(title: const Text('Countersign')),
			body: const Center(
				child: Text('HSM-anchored countersignature stub'),
			),
		);
	}
}
