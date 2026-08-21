import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class KioskScreen extends ConsumerWidget {
	const KioskScreen({super.key});

	@override
	Widget build(BuildContext context, WidgetRef ref) {
		return Scaffold(
			appBar: AppBar(title: const Text('Kiosk Mode')),
			body: const Center(child: Text('Kiosk mode stub')),
		);
	}
}
