import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class TallyVerifyScreen extends ConsumerWidget {
	const TallyVerifyScreen({super.key});

	@override
	Widget build(BuildContext context, WidgetRef ref) {
		return Scaffold(
			appBar: AppBar(title: const Text('Tally Verification')),
			body: const Center(child: Text('Sample-based tally verification stub')),
		);
	}
}
